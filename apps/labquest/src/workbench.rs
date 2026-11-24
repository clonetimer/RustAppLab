//! Native, offline browser interface for existing trusted learner projects.
mod book_projects;
mod experiments;
mod http;
mod jobs;
mod model;

use http::{Fields, Response};
use model::{json_string, Workspace};
use std::ffi::OsString;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const INDEX: &str = include_str!("workbench/ui/index.html");
const STYLE: &str = include_str!("workbench/ui/app.css");
const SCRIPT: &str = include_str!("workbench/ui/app.js");
const KNOWLEDGE_SCRIPT: &str = include_str!("workbench/ui/knowledge.js");
const EXPERIMENT_SCRIPT: &str = include_str!("workbench/ui/experiments.js");
const HELP:&str="Rust AppLab 网页学习工作台\n\n用法：labquest workbench [--workspace 现有学习目录] [--port 0] [--open]\n默认使用当前目录，端口 0 自动选择空闲端口；仅监听 127.0.0.1。\n在同一网页阅读任务卡、编辑自己的代码、运行真实检查和查看结果。\nFirstCard、12 款 Studio 与 D1–D6 七个专题已接入；Cargo 与课程原有系统依赖仍需安装。\n--open 打开默认浏览器；关闭本窗口或按 Ctrl+C 停止本地服务。\n";

#[derive(Debug)]
pub(super) struct ApiError {
    pub status: u16,
    pub message: String,
}
impl ApiError {
    pub fn new(status: u16, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

struct App {
    workspace: Mutex<Workspace>,
    jobs: Mutex<jobs::Jobs>,
    experiments: Mutex<experiments::Store>,
}

fn error_response(error: ApiError) -> Response {
    Response::json(
        error.status,
        format!("{{\"error\":{}}}", json_string(&error.message)),
    )
}

fn step(fields: &Fields) -> Result<usize, ApiError> {
    fields
        .get("step")
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| ApiError::new(400, "步骤编号不正确"))
        })
        .unwrap_or(Ok(1))
}

fn route(app: &Arc<App>, request: http::Request) -> Result<Response, ApiError> {
    let (path, query) = request
        .target
        .split_once('?')
        .unwrap_or((&request.target, ""));
    if request.method == "GET" {
        let fields = http::fields(query.as_bytes())?;
        match path {
            "/" => Ok(Response::new(
                200,
                "text/html; charset=utf-8",
                INDEX.as_bytes().to_vec(),
            )),
            "/assets/app.css" => Ok(Response::new(
                200,
                "text/css; charset=utf-8",
                STYLE.as_bytes().to_vec(),
            )),
            "/assets/app.js" => Ok(Response::new(
                200,
                "text/javascript; charset=utf-8",
                SCRIPT.as_bytes().to_vec(),
            )),
            "/assets/knowledge.js" => Ok(Response::new(
                200,
                "text/javascript; charset=utf-8",
                KNOWLEDGE_SCRIPT.as_bytes().to_vec(),
            )),
            "/assets/experiments.js" => Ok(Response::new(
                200,
                "text/javascript; charset=utf-8",
                EXPERIMENT_SCRIPT.as_bytes().to_vec(),
            )),
            "/api/health" => Ok(Response::json(
                200,
                format!(
                    "{{\"status\":\"ok\",\"app\":\"rustapplab-workbench\",\"version\":{}}}",
                    json_string(env!("CARGO_PKG_VERSION"))
                ),
            )),
            "/api/book" => {
                http::only(&fields, &["id"])?;
                let id = http::required(&fields, "id")?;
                let (title, markdown) = crate::book_reading::workbench_topic(id)
                    .ok_or_else(|| ApiError::new(404, "未找到知识专题"))?;
                Ok(Response::json(
                    200,
                    format!(
                        "{{\"title\":{},\"markdown\":{}}}",
                        json_string(title),
                        json_string(markdown)
                    ),
                ))
            }
            "/api/experiment" => {
                http::only(&fields, &["code"])?;
                Ok(Response::json(
                    200,
                    experiments::material(http::required(&fields, "code")?)?,
                ))
            }
            "/api/experiment-job" => {
                http::only(&fields, &["id"])?;
                let id = http::required(&fields, "id")?
                    .parse::<u64>()
                    .map_err(|_| ApiError::new(400, "实验编号不正确"))?;
                Ok(Response::json(200, experiments::get(app, id)?.json()))
            }
            "/api/job" => {
                http::only(&fields, &["id"])?;
                let id = http::required(&fields, "id")?
                    .parse::<u64>()
                    .map_err(|_| ApiError::new(400, "操作编号不正确"))?;
                Ok(Response::json(200, jobs::get(app, id)?.json()))
            }
            "/api/catalog" | "/api/course" | "/api/file" | "/api/lesson" => {
                let workspace = app
                    .workspace
                    .lock()
                    .map_err(|_| ApiError::new(500, "工作区状态不可用"))?;
                let value = match path {
                    "/api/catalog" => {
                        http::only(&fields, &[])?;
                        workspace.catalog_json()?
                    }
                    "/api/course" => {
                        http::only(&fields, &["id", "step"])?;
                        workspace.course_json(http::required(&fields, "id")?, step(&fields)?)?
                    }
                    "/api/file" => {
                        http::only(&fields, &["id", "path", "project_path"])?;
                        if let Some(expected) = fields.get("project_path") {
                            let selected =
                                workspace.project_path(http::required(&fields, "id")?)?;
                            if selected.to_string_lossy() != expected.as_str() {
                                return Err(ApiError::new(
                                    409,
                                    "此课程已在另一个页面打开其他工程；请重新选择工程后读取文件",
                                ));
                            }
                        }
                        workspace.file_json(
                            http::required(&fields, "id")?,
                            http::required(&fields, "path")?,
                        )?
                    }
                    "/api/lesson" => {
                        http::only(&fields, &["id"])?;
                        return Ok(Response::lesson(
                            workspace.lesson_html(http::required(&fields, "id")?)?,
                        ));
                    }
                    _ => unreachable!(),
                };
                Ok(Response::json(200, value))
            }
            _ => Err(ApiError::new(404, "页面不存在")),
        }
    } else {
        if !query.is_empty() {
            return Err(ApiError::new(400, "操作参数请放在表单中"));
        }
        let fields = http::fields(&request.body)?;
        if path == "/api/experiment" {
            http::only(&fields, &["code", "variant"])?;
            return Ok(Response::json(
                202,
                experiments::start(
                    app,
                    http::required(&fields, "code")?,
                    http::required(&fields, "variant")?,
                )?
                .json(),
            ));
        }
        if path == "/api/action" {
            http::only(&fields, &["id", "action", "step", "args", "project_path"])?;
            let job = jobs::start(
                app,
                http::required(&fields, "id")?,
                http::required(&fields, "action")?,
                step(&fields)?,
                fields.get("args").map(String::as_str).unwrap_or(""),
                fields.get("project_path").map(String::as_str),
            )?;
            return Ok(Response::json(202, job.json()));
        }
        if !matches!(path, "/api/create" | "/api/open" | "/api/save") {
            return Err(ApiError::new(404, "操作不存在"));
        }
        // Lock order is always jobs -> workspace; capture neither lock while running a child.
        let jobs = app
            .jobs
            .lock()
            .map_err(|_| ApiError::new(500, "操作状态不可用"))?;
        jobs.ensure_idle()?;
        let mut workspace = app
            .workspace
            .lock()
            .map_err(|_| ApiError::new(500, "工作区状态不可用"))?;
        let json = match path {
            "/api/create" => {
                http::only(&fields, &["id"])?;
                workspace.create(http::required(&fields, "id")?)?
            }
            "/api/open" => {
                http::only(&fields, &["id", "path"])?;
                workspace.open(
                    http::required(&fields, "id")?,
                    Path::new(http::required(&fields, "path")?),
                )?
            }
            "/api/save" => {
                http::only(
                    &fields,
                    &["id", "path", "content", "revision", "project_path"],
                )?;
                if let Some(expected) = fields.get("project_path") {
                    let selected = workspace.project_path(http::required(&fields, "id")?)?;
                    if selected.to_string_lossy() != expected.as_str() {
                        return Err(ApiError::new(
                            409,
                            "此课程已在另一个页面打开其他工程；编辑内容已保留，请重新选择工程",
                        ));
                    }
                }
                workspace.save(
                    http::required(&fields, "id")?,
                    http::required(&fields, "path")?,
                    http::required(&fields, "content")?,
                    http::required(&fields, "revision")?,
                )?
            }
            _ => unreachable!(),
        };
        Ok(Response::json(200, json))
    }
}

struct ConnectionCount(Arc<AtomicUsize>);
impl Drop for ConnectionCount {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

fn connection(mut stream: TcpStream, app: &Arc<App>, port: u16) {
    let response = http::read(&mut stream, port)
        .and_then(|request| route(app, request))
        .unwrap_or_else(error_response);
    let _ = response.write(&mut stream);
}

fn open_browser(url: &str) {
    let mut command = if cfg!(windows) {
        let mut command = Command::new("rundll32.exe");
        command.arg("url.dll,FileProtocolHandler");
        command
    } else {
        Command::new("xdg-open")
    };
    match command
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(mut child) => {
            let _ = std::thread::Builder::new()
                .name("open-learning-browser".into())
                .spawn(move || {
                    if let Ok(status) = child.wait() {
                        if !status.success() {
                            eprintln!("浏览器未自动打开，请复制上面的本机地址。")
                        }
                    }
                });
        }
        Err(_) => eprintln!("浏览器未自动打开，请复制上面的本机地址。"),
    }
}

pub fn run(args: &[OsString]) -> Result<(), String> {
    if args.len() == 1 && matches!(args[0].to_str(), Some("--help" | "help" | "-h")) {
        print!("{HELP}");
        return Ok(());
    }
    let mut root = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut port = 0u16;
    let mut open = false;
    let mut seen = std::collections::BTreeSet::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let arg = arg.to_str().ok_or("工作台选项名称不正确")?;
        if !seen.insert(arg) {
            return Err(format!("选项重复：{arg}"));
        }
        match arg {
            "--workspace" => {
                root = PathBuf::from(iter.next().ok_or("--workspace 需要一个学习目录")?)
            }
            "--port" => {
                port = iter
                    .next()
                    .and_then(|v| v.to_str())
                    .and_then(|v| v.parse().ok())
                    .ok_or("--port 需要 0–65535 的端口编号")?
            }
            "--open" => open = true,
            _ => return Err(format!("未知工作台选项：{arg}\n{HELP}")),
        }
    }
    let workspace = Workspace::new(root).map_err(|e| e.message)?;
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))
        .map_err(|e| format!("无法启动本机工作台：{e}。可使用 --port 0 自动选择端口。"))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let url = format!("http://127.0.0.1:{port}/");
    println!("Rust AppLab 网页学习工作台\n学习目录：{}\nRUSTAPPLAB_WORKBENCH_URL={url}\n保持此窗口打开；按 Ctrl+C 停止服务。",workspace.root.display());
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    let app = Arc::new(App {
        workspace: Mutex::new(workspace),
        jobs: Mutex::new(jobs::Jobs::new()),
        experiments: Mutex::new(experiments::Store::new()),
    });
    if open {
        open_browser(&url);
    }
    let active = Arc::new(AtomicUsize::new(0));
    for incoming in listener.incoming() {
        let mut stream = match incoming {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("连接失败：{error}");
                continue;
            }
        };
        if active.fetch_add(1, Ordering::Relaxed) >= 16 {
            active.fetch_sub(1, Ordering::Relaxed);
            let _ = stream.set_write_timeout(Some(std::time::Duration::from_millis(500)));
            let _ = error_response(ApiError::new(503, "连接较多，请稍后重试")).write(&mut stream);
            continue;
        }
        let guard = ConnectionCount(Arc::clone(&active));
        let shared = Arc::clone(&app);
        if let Err(error) = std::thread::Builder::new()
            .name("learning-http".into())
            .spawn(move || {
                let _guard = guard;
                connection(stream, &shared, port);
            })
        {
            eprintln!("无法处理连接：{error}");
        }
    }
    Ok(())
}
