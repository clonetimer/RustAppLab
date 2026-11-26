//! E1-B: actual SQLite transactions and bounded observability for the E1-A HTTPS API.
//! The local teaching service is not a production identity or multi-tenant system.
use axum::{
    Router, Json,
    extract::{DefaultBodyLimit, Path as AxumPath, Request, State},
    http::{HeaderMap, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use e1_secure_lab::{Credentials, protect_router};
use rusqlite::{Connection, OpenFlags, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque, fs, path::Path, sync::{Arc, Mutex}, time::{Duration, Instant}
};
use tokio::{sync::{mpsc, oneshot}, task::JoinHandle};

const APP_ID: i64 = 0x52414c31;
const VERSION: i64 = 1;
pub const MAX_BATCH: usize = 16;
pub const QUEUE_CAPACITY: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ticket {
    pub id: u64,
    pub title: String,
    pub done: bool,
}

impl From<servicedesk::Ticket> for Ticket {
    fn from(ticket: servicedesk::Ticket) -> Self {
        Self { id: ticket.id, title: ticket.title, done: ticket.done }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Invalid, NotFound, Conflict, Busy, Closed, Storage, NotImplemented,
}

impl Error {
    fn domain(message: String) -> Self {
        if message.contains("编号不存在") { Self::NotFound }
        else if message.contains("已关闭") { Self::Conflict }
        else { Self::Invalid }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(_: rusqlite::Error) -> Self { Self::Storage }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            Self::Invalid => (StatusCode::UNPROCESSABLE_ENTITY, "invalid"),
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            Self::Conflict => (StatusCode::CONFLICT, "conflict"),
            Self::Busy => (StatusCode::SERVICE_UNAVAILABLE, "busy"),
            Self::Closed => (StatusCode::SERVICE_UNAVAILABLE, "closed"),
            Self::Storage => (StatusCode::SERVICE_UNAVAILABLE, "storage"),
            Self::NotImplemented => (StatusCode::NOT_IMPLEMENTED, "todo"),
        };
        (status, Json(serde_json::json!({"error": code}))).into_response()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub sequence: i64,
    pub operation: String,
    pub ticket_id: u64,
}

/// Fixed SQLite infrastructure: create only a new database, never auto-create on open.
/// Path must be in a trusted local directory. No hostile-directory race guarantee.
pub struct Store { connection: Connection }

impl Store {
    pub fn init(path: &Path) -> Result<(), Error> {
        let reservation = fs::OpenOptions::new().write(true).create_new(true)
            .open(path).map_err(|_| Error::Storage)?;
        drop(reservation);
        let result = (|| {
            let mut connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
            connection.busy_timeout(Duration::from_secs(1))?;
            connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL;")?;
            let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            transaction.execute_batch(
                "CREATE TABLE tickets(
                    id INTEGER PRIMARY KEY CHECK(id>0),
                    title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 120),
                    done INTEGER NOT NULL CHECK(done IN (0,1))
                );
                CREATE TABLE audit_events(
                    sequence INTEGER PRIMARY KEY,
                    operation TEXT NOT NULL CHECK(operation IN ('create','close')),
                    ticket_id INTEGER NOT NULL REFERENCES tickets(id)
                );
                PRAGMA application_id=1380011057;
                PRAGMA user_version=1;"
            )?;
            transaction.commit()?;
            Ok::<(), Error>(())
        })();
        if result.is_err() {
            // Reservation succeeded, so this path belongs to this initialization.
            let _ = fs::remove_file(path);
        }
        result
    }

    pub fn open(path: &Path) -> Result<Self, Error> {
        let metadata = fs::symlink_metadata(path).map_err(|_| Error::Storage)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() { return Err(Error::Storage); }
        let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        connection.busy_timeout(Duration::from_secs(1))?;
        connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL;")?;
        let app: i64 = connection.query_row("PRAGMA application_id", [], |r| r.get(0))?;
        let version: i64 = connection.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if app != APP_ID || version != VERSION { return Err(Error::Storage); }
        let store = Self { connection };
        store.list()?;
        store.audit()?;
        Ok(store)
    }

    pub fn list(&self) -> Result<Vec<Ticket>, Error> {
        Ok(load(&self.connection)?.into_iter().map(Ticket::from).collect())
    }

    pub fn audit(&self) -> Result<Vec<AuditEvent>, Error> {
        let mut statement = self.connection.prepare(
            "SELECT sequence, operation, ticket_id FROM audit_events ORDER BY sequence"
        )?;
        let rows = statement.query_map([], |r| Ok(AuditEvent {
            sequence: r.get(0)?, operation: r.get(1)?, ticket_id: r.get::<_, i64>(2)? as u64
        }))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}

fn load(connection: &Connection) -> Result<Vec<servicedesk::Ticket>, Error> {
    let mut statement = connection.prepare("SELECT id,title,done FROM tickets ORDER BY id")?;
    let rows = statement.query_map([], |r| Ok(servicedesk::Ticket {
        id: r.get::<_, i64>(0)? as u64, title: r.get(1)?, done: r.get(2)?
    }))?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn insert_audit(transaction: &rusqlite::Transaction<'_>, operation: &str, id: u64) -> Result<(), Error> {
    transaction.execute("INSERT INTO audit_events(operation,ticket_id) VALUES(?1,?2)",
                        params![operation, id as i64])?;
    Ok(())
}

impl Store {
    /// Atomic bounded batch, including all audit records. Reuse the real C2 domain rules.
    pub fn create_batch(&mut self, titles: &[String]) -> Result<Vec<Ticket>, Error> {
        if titles.is_empty() || titles.len() > MAX_BATCH { return Err(Error::Invalid); }
        let transaction = self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut current = load(&transaction)?;
        let mut saved = Vec::new();
        for input in titles {
            let ticket = servicedesk::apply(&mut current, servicedesk::Action::Add(input.clone()))
                .map_err(Error::domain)?;
            transaction.execute("INSERT INTO tickets(id,title,done) VALUES(?1,?2,?3)",
                params![ticket.id as i64, ticket.title.as_str(), ticket.done])?;
            insert_audit(&transaction, "create", ticket.id)?;
            saved.push(ticket.into());
        }
        transaction.commit()?;
        Ok(saved)
    }

    pub fn close(&mut self, id: u64) -> Result<Ticket, Error> {
        let transaction = self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut current = load(&transaction)?;
        let ticket = servicedesk::apply(&mut current, servicedesk::Action::Close(id))
            .map_err(Error::domain)?;
        let changed = transaction.execute("UPDATE tickets SET done=1 WHERE id=?1 AND done=0",
                                          params![ticket.id as i64])?;
        if changed != 1 { return Err(Error::Storage); }
        insert_audit(&transaction, "close", ticket.id)?;
        transaction.commit()?;
        Ok(ticket.into())
    }
}

/// A full/closed queue has an explicit outcome instead of silently dropping work.
pub fn enqueue<T>(sender: &mpsc::Sender<T>, command: T) -> Result<(), Error> {
    sender.try_send(command).map_err(|failure| match failure {
        mpsc::error::TrySendError::Full(_) => Error::Busy,
        mpsc::error::TrySendError::Closed(_) => Error::Closed,
    })
}

enum Command {
    List(oneshot::Sender<Result<Vec<Ticket>, Error>>),
    Create(String, oneshot::Sender<Result<Ticket, Error>>),
    Close(u64, oneshot::Sender<Result<Ticket, Error>>),
}

#[derive(Clone)]
pub struct Backend { sender: mpsc::Sender<Command> }

impl Backend {
    pub async fn list(&self) -> Result<Vec<Ticket>, Error> {
        let (reply, response) = oneshot::channel();
        enqueue(&self.sender, Command::List(reply))?;
        response.await.map_err(|_| Error::Closed)?
    }

    pub async fn create(&self, title: String) -> Result<Ticket, Error> {
        let (reply, response) = oneshot::channel();
        enqueue(&self.sender, Command::Create(title, reply))?;
        response.await.map_err(|_| Error::Closed)?
    }

    pub async fn close(&self, id: u64) -> Result<Ticket, Error> {
        let (reply, response) = oneshot::channel();
        enqueue(&self.sender, Command::Close(id, reply))?;
        response.await.map_err(|_| Error::Closed)?
    }
}

/// SQLite I/O happens on a single blocking actor, not a Tokio I/O worker.
/// Drop every Backend clone and await the actor to drain already admitted work.
/// Client disconnection after enqueue does not imply rollback.
pub fn start_store(path: &Path) -> Result<(Backend, JoinHandle<()>), Error> {
    let mut store = Store::open(path)?;
    let runtime = tokio::runtime::Handle::try_current().map_err(|_| Error::Closed)?;
    let (sender, mut receiver) = mpsc::channel(QUEUE_CAPACITY);
    let worker = runtime.spawn_blocking(move || {
        while let Some(command) = receiver.blocking_recv() {
            match command {
                Command::List(reply) => { let _ = reply.send(store.list()); }
                Command::Create(title, reply) => {
                    let result = store.create_batch(&[title]).and_then(|mut rows| rows.pop().ok_or(Error::Storage));
                    let _ = reply.send(result);
                }
                Command::Close(id, reply) => { let _ = reply.send(store.close(id)); }
            }
        }
    });
    Ok((Backend { sender }, worker))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessEvent {
    pub sequence: u64,
    pub method: String,
    pub route: String,
    pub status: u16,
    pub elapsed_ms: u64,
}

struct AccessState {
    total: u64,
    dropped: u64,
    entries: VecDeque<AccessEvent>,
}

#[derive(Clone)]
pub struct AccessLog {
    state: Arc<Mutex<AccessState>>,
    capacity: usize,
}

impl AccessLog {
    pub fn new(capacity: usize) -> Result<Self, Error> {
        if !(1..=256).contains(&capacity) { return Err(Error::Invalid); }
        Ok(Self { state: Arc::new(Mutex::new(AccessState { total: 0, dropped: 0,
             entries: VecDeque::with_capacity(capacity) })), capacity })
    }

    pub fn snapshot(&self) -> (u64, u64, Vec<AccessEvent>) {
        let state = self.state.lock().expect("access log lock");
        (state.total, state.dropped, state.entries.iter().cloned().collect())
    }

    pub fn json_lines(&self) -> String {
        let (_, _, entries) = self.snapshot();
        entries.into_iter().map(|event| serde_json::to_string(&event).expect("fixed JSON event"))
            .collect::<Vec<_>>().join("\n")
    }
}

impl AccessLog {
    /// Use only a bounded, fixed vocabulary, never raw URL, body, headers or tokens.
    pub fn record(&self, method: &Method, path: &str, status: StatusCode, elapsed_ms: u64) -> u64 {
        let method = match *method { Method::GET => "GET", Method::POST => "POST", _ => "OTHER" };
        let route = match path {
            "/health" => "health",
            "/api/tickets" => "tickets",
            value if value.strip_prefix("/api/tickets/").and_then(|s| s.strip_suffix("/close"))
                .is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()) && id.parse::<u64>().is_ok()) => "ticket_close",
            _ => "other",
        };
        let mut state = self.state.lock().expect("access log lock");
        state.total = state.total.saturating_add(1);
        let sequence = state.total;
        if state.entries.len() == self.capacity {
            state.entries.pop_front();
            state.dropped = state.dropped.saturating_add(1);
        }
        state.entries.push_back(AccessEvent {
            sequence, method: method.to_owned(), route: route.to_owned(),
            status: status.as_u16(), elapsed_ms,
        });
        sequence
    }
}

async fn observe(State(log): State<AccessLog>, request: Request, next: Next) -> Response {
    let method = request.method().clone();
    // Classify the path inside record; no query, headers or body is passed to the log.
    let path = request.uri().path().to_owned();
    let start = Instant::now();
    let mut response = next.run(request).await;
    let millis = start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let sequence = log.record(&method, &path, response.status(), millis);
    if let Ok(value) = axum::http::HeaderValue::from_str(&sequence.to_string()) {
        response.headers_mut().insert("x-e1-request-id", value);
    }
    response
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Create { title: String }

fn write_allowed(headers: &HeaderMap) -> bool {
    let mut markers = headers.get_all("x-rustapplab-client").iter();
    markers.next().and_then(|v| v.to_str().ok()) == Some("1") &&
        markers.next().is_none() && !headers.contains_key(axum::http::header::ORIGIN)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok":true,"service":"E1-B"}))
}
async fn list(State(backend): State<Backend>) -> Result<Json<serde_json::Value>, Error> {
    let tickets = backend.list().await?;
    Ok(Json(serde_json::json!({"count": tickets.len(), "tickets": tickets})))
}
async fn create(State(backend): State<Backend>, headers: HeaderMap, Json(input): Json<Create>)
    -> Result<Response, Error>
{
    if !write_allowed(&headers) { return Ok(StatusCode::FORBIDDEN.into_response()); }
    Ok((StatusCode::CREATED, Json(backend.create(input.title).await?)).into_response())
}
async fn close(State(backend): State<Backend>, headers: HeaderMap, AxumPath(id): AxumPath<u64>)
    -> Result<Response, Error>
{
    if !write_allowed(&headers) { return Ok(StatusCode::FORBIDDEN.into_response()); }
    Ok(Json(backend.close(id).await?).into_response())
}

/// Replace C2's persistence implementation without changing E1-A TLS or authorization.
pub fn router(backend: Backend, credentials: Credentials, log: AccessLog) -> Router {
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/tickets", get(list).post(create))
        .route("/api/tickets/{id}/close", post(close))
        .layer(DefaultBodyLimit::max(2048))
        .with_state(backend);
    // The outer observer sees both successful and rejected requests.
    protect_router(app, credentials).layer(middleware::from_fn_with_state(log, observe))
}
