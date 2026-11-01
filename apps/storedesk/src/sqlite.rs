//! Small, deliberately restricted SQLite adapter supplied to the learner.
//! Linux/macOS: links the system sqlite3 library. Windows MSVC: Windows SDK winsqlite3.
//! Not a replacement for a general-purpose database crate; no callbacks, blobs, or extensions.
//! Text bindings use SQLITE_STATIC with owned CString buffers retained until finalize.
//! All FFI pointers stay private; the learner uses execute/query/transaction, not unsafe.
use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    os::raw::{c_char, c_int, c_void},
    path::Path,
    ptr::NonNull,
    rc::Rc,
};

#[repr(C)]
struct RawDb {
    _private: [u8; 0],
}
#[repr(C)]
struct RawStmt {
    _private: [u8; 0],
}
#[cfg_attr(target_os = "windows", link(name = "winsqlite3"))]
#[cfg_attr(not(target_os = "windows"), link(name = "sqlite3"))]
extern "C" {
    fn sqlite3_open_v2(
        name: *const c_char,
        out: *mut *mut RawDb,
        flags: c_int,
        vfs: *const c_char,
    ) -> c_int;
    fn sqlite3_close_v2(db: *mut RawDb) -> c_int;
    fn sqlite3_errmsg(db: *mut RawDb) -> *const c_char;
    fn sqlite3_busy_timeout(db: *mut RawDb, millis: c_int) -> c_int;
    fn sqlite3_prepare_v2(
        db: *mut RawDb,
        sql: *const c_char,
        length: c_int,
        out: *mut *mut RawStmt,
        tail: *mut *const c_char,
    ) -> c_int;
    fn sqlite3_finalize(stmt: *mut RawStmt) -> c_int;
    fn sqlite3_bind_parameter_count(stmt: *mut RawStmt) -> c_int;
    fn sqlite3_bind_int64(stmt: *mut RawStmt, index: c_int, value: i64) -> c_int;
    fn sqlite3_bind_text(
        stmt: *mut RawStmt,
        index: c_int,
        value: *const c_char,
        length: c_int,
        destroy: Option<unsafe extern "C" fn(*mut c_void)>,
    ) -> c_int;
    fn sqlite3_step(stmt: *mut RawStmt) -> c_int;
    fn sqlite3_column_count(stmt: *mut RawStmt) -> c_int;
    fn sqlite3_column_type(stmt: *mut RawStmt, column: c_int) -> c_int;
    fn sqlite3_column_int64(stmt: *mut RawStmt, column: c_int) -> i64;
    fn sqlite3_column_text(stmt: *mut RawStmt, column: c_int) -> *const u8;
    fn sqlite3_column_bytes(stmt: *mut RawStmt, column: c_int) -> c_int;
    fn sqlite3_changes(db: *mut RawDb) -> c_int;
    fn sqlite3_stmt_readonly(stmt: *mut RawStmt) -> c_int;
    fn sqlite3_libversion() -> *const c_char;
}
const OK: c_int = 0;
const ROW: c_int = 100;
const DONE: c_int = 101;
pub type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Copy)]
pub enum Param<'a> {
    Int(i64),
    Text(&'a str),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cell {
    Int(i64),
    Text(String),
    Null,
}
impl Cell {
    pub fn int(&self) -> Result<i64> {
        match self {
            Self::Int(n) => Ok(*n),
            _ => Err("数据库字段不是整数".into()),
        }
    }
    pub fn text(&self) -> Result<&str> {
        match self {
            Self::Text(s) => Ok(s),
            _ => Err("数据库字段不是文本".into()),
        }
    }
}
/// Not Send or Sync. Only one thread owns a connection; transactions require &mut self.
pub struct Db {
    ptr: NonNull<RawDb>,
    _not_send: PhantomData<Rc<()>>,
}
impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        // Absolute names prevent magic names such as :memory: or file: from being interpreted.
        if path.as_os_str().is_empty() {
            return Err("数据库路径不能为空".into());
        }
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| e.to_string())?
                .join(path)
        };
        Self::open_name(
            absolute
                .to_str()
                .ok_or("此教学适配器只支持 UTF-8 数据库路径")?,
        )
    }
    pub fn memory() -> Result<Self> {
        Self::open_name(":memory:")
    }
    fn open_name(name: &str) -> Result<Self> {
        let name = CString::new(name).map_err(|_| "数据库路径含 NUL")?;
        let mut raw = std::ptr::null_mut();
        // SAFETY: name is NUL-terminated; raw is a valid output slot; NULL vfs selects default.
        let code = unsafe {
            sqlite3_open_v2(
                name.as_ptr(),
                &mut raw,
                0x0002 | 0x0004 | 0x10000,
                std::ptr::null(),
            )
        };
        let ptr = NonNull::new(raw).ok_or_else(|| format!("SQLite({code}) 无法分配连接"))?;
        let db = Self {
            ptr,
            _not_send: PhantomData,
        };
        db.result(code)?; // Drop closes even a failed but allocated connection.
                          // SAFETY: ptr is a live connection owned by db.
        db.result(unsafe { sqlite3_busy_timeout(db.ptr.as_ptr(), 1000) })?;
        Ok(db)
    }
    fn result(&self, code: c_int) -> Result<()> {
        if code == OK {
            return Ok(());
        }
        // SAFETY: db is live; error message is owned by SQLite and copied before any next call.
        let ptr = unsafe { sqlite3_errmsg(self.ptr.as_ptr()) };
        let message = if ptr.is_null() {
            "未知 SQLite 错误".into()
        } else {
            unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned()
        };
        Err(format!("SQLite({code}): {message}"))
    }
    fn prepare<'a>(&'a self, sql: &str, params: &[Param<'_>]) -> Result<Statement<'a>> {
        let sql = CString::new(sql).map_err(|_| "SQL 含 NUL")?;
        let mut raw = std::ptr::null_mut();
        let mut tail = std::ptr::null();
        // SAFETY: sql is a valid C string; output pointers refer to initialized local storage.
        let code =
            unsafe { sqlite3_prepare_v2(self.ptr.as_ptr(), sql.as_ptr(), -1, &mut raw, &mut tail) };
        if code != OK {
            if !raw.is_null() {
                unsafe {
                    sqlite3_finalize(raw);
                }
            }
            self.result(code)?;
        }
        let ptr = NonNull::new(raw).ok_or("SQL 不能为空或只有注释")?;
        let mut stmt = Statement {
            db: self,
            ptr,
            strings: Vec::with_capacity(params.len()),
        };
        // SAFETY: tail points within the still-live sql CString (or is null); SQLite supplies it.
        if !tail.is_null()
            && !unsafe { CStr::from_ptr(tail) }
                .to_string_lossy()
                .trim()
                .is_empty()
        {
            return Err("一次只能执行一条 SQL，不接受拼接的多条语句".into());
        }
        // SAFETY: statement is live and not yet stepped.
        let count = unsafe { sqlite3_bind_parameter_count(ptr.as_ptr()) };
        if usize::try_from(count).ok() != Some(params.len()) {
            return Err("SQL 占位符数量与参数数量不同".into());
        }
        for (index, value) in params.iter().enumerate() {
            let index = c_int::try_from(index + 1).map_err(|_| "SQL 参数过多")?;
            let code = match value {
                Param::Int(n) => unsafe { sqlite3_bind_int64(ptr.as_ptr(), index, *n) },
                Param::Text(text) => {
                    let length = c_int::try_from(text.len()).map_err(|_| "SQL 文本参数过长")?;
                    let text = CString::new(*text).map_err(|_| "SQL 文本参数含 NUL")?;
                    stmt.strings.push(text);
                    let stored = stmt.strings.last().expect("just pushed");
                    // SAFETY: CString heap buffer does not move when Vec<CString> moves.
                    // None means SQLITE_STATIC. Strings remain alive through Statement::drop/finalize.
                    unsafe { sqlite3_bind_text(ptr.as_ptr(), index, stored.as_ptr(), length, None) }
                }
            };
            self.result(code)?;
        }
        Ok(stmt)
    }
    pub fn execute(&self, sql: &str, params: &[Param<'_>]) -> Result<usize> {
        let stmt = self.prepare(sql, params)?;
        // SAFETY: live prepared statement; no rows have been stepped yet.
        if unsafe { sqlite3_column_count(stmt.ptr.as_ptr()) } != 0 {
            return Err("有结果列的 SQL 请使用 query".into());
        }
        let code = unsafe { sqlite3_step(stmt.ptr.as_ptr()) };
        if code != DONE {
            self.result(code)?;
            return Err("SQL 未完成".into());
        }
        usize::try_from(unsafe { sqlite3_changes(self.ptr.as_ptr()) })
            .map_err(|_| "非法受影响行数".into())
    }
    pub fn query(&self, sql: &str, params: &[Param<'_>]) -> Result<Vec<Vec<Cell>>> {
        let stmt = self.prepare(sql, params)?;
        // Query is intentionally restricted to read-only statements.
        if unsafe { sqlite3_stmt_readonly(stmt.ptr.as_ptr()) } == 0 {
            return Err("query 只接受只读 SQL".into());
        }
        let columns = unsafe { sqlite3_column_count(stmt.ptr.as_ptr()) };
        if !(1..=32).contains(&columns) {
            return Err("查询应有 1–32 列".into());
        }
        let mut rows = Vec::new();
        loop {
            let code = unsafe { sqlite3_step(stmt.ptr.as_ptr()) };
            if code == DONE {
                return Ok(rows);
            }
            if code != ROW {
                self.result(code)?;
                return Err("读取 SQL 行失败".into());
            }
            if rows.len() >= 10000 {
                return Err("教学查询上限 10000 行，请加 LIMIT".into());
            }
            let mut row = Vec::new();
            for column in 0..columns {
                // SAFETY: code == ROW and column is within column_count; values are copied now.
                let cell = match unsafe { sqlite3_column_type(stmt.ptr.as_ptr(), column) } {
                    1 => Cell::Int(unsafe { sqlite3_column_int64(stmt.ptr.as_ptr(), column) }),
                    3 => {
                        let ptr = unsafe { sqlite3_column_text(stmt.ptr.as_ptr(), column) };
                        let length = unsafe { sqlite3_column_bytes(stmt.ptr.as_ptr(), column) };
                        if ptr.is_null() || !(0..=4 * 1024 * 1024).contains(&length) {
                            return Err("SQLite 文本为空指针、过大或分配失败".into());
                        }
                        let bytes = unsafe { std::slice::from_raw_parts(ptr, length as usize) };
                        Cell::Text(
                            std::str::from_utf8(bytes)
                                .map_err(|_| "数据库文本不是 UTF-8")?
                                .into(),
                        )
                    }
                    5 => Cell::Null,
                    _ => return Err("教学适配器只支持整数、文本与 NULL".into()),
                };
                row.push(cell);
            }
            rows.push(row);
        }
    }
    /// BEGIN IMMEDIATE plus a rollback guard. Any returned error or unwinding rolls back.
    pub fn transaction<T>(&mut self, work: impl FnOnce(&Db) -> Result<T>) -> Result<T> {
        self.execute("BEGIN IMMEDIATE", &[])?;
        let mut guard = Rollback {
            db: self,
            active: true,
        };
        match work(guard.db) {
            Ok(value) => {
                guard.db.execute("COMMIT", &[])?;
                guard.active = false;
                Ok(value)
            }
            Err(error) => match guard.db.execute("ROLLBACK", &[]) {
                Ok(_) => {
                    guard.active = false;
                    Err(error)
                }
                Err(rollback) => Err(format!(
                    "{error}; 回滚也失败：{rollback}；请关闭连接并检查数据库"
                )),
            },
        }
    }
}
impl Drop for Db {
    fn drop(&mut self) {
        // SAFETY: this is the unique owner; all Statement borrows have ended before drop.
        unsafe {
            sqlite3_close_v2(self.ptr.as_ptr());
        }
    }
}
struct Statement<'a> {
    db: &'a Db,
    ptr: NonNull<RawStmt>,
    strings: Vec<CString>,
}
impl Drop for Statement<'_> {
    fn drop(&mut self) {
        let _keep_connection_alive = self.db;
        // SAFETY: sole statement owner. Finalize runs BEFORE the strings field is dropped.
        unsafe {
            sqlite3_finalize(self.ptr.as_ptr());
        }
    }
}
struct Rollback<'a> {
    db: &'a Db,
    active: bool,
}
impl Drop for Rollback<'_> {
    fn drop(&mut self) {
        if self.active {
            let _ = self.db.execute("ROLLBACK", &[]);
        }
    }
}
pub fn version() -> String {
    // SAFETY: SQLite supplies a static NUL-terminated version string.
    unsafe { CStr::from_ptr(sqlite3_libversion()) }
        .to_string_lossy()
        .into_owned()
}
