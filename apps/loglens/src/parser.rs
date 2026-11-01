//! LogLens 支持两种有明确契约的单行日志格式，不猜测任意日志文本。
//! 1) `INFO service started`
//! 2) `2026-10-28T10:20:30Z ERROR request failed`
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl Level {
    pub const ALL: [Self; 6] = [
        Self::Trace,
        Self::Debug,
        Self::Info,
        Self::Warn,
        Self::Error,
        Self::Fatal,
    ];

    pub fn parse(raw: &str) -> Option<Self> {
        match raw
            .trim_matches(|c| c == '[' || c == ']' || c == ':')
            .to_ascii_uppercase()
            .as_str()
        {
            "TRACE" => Some(Self::Trace),
            "DEBUG" => Some(Self::Debug),
            "INFO" => Some(Self::Info),
            "WARN" | "WARNING" => Some(Self::Warn),
            "ERROR" | "ERR" => Some(Self::Error),
            "FATAL" | "CRITICAL" => Some(Self::Fatal),
            _ => None,
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Fatal => "FATAL",
        };
        f.write_str(text)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct LogRecord {
    pub timestamp: Option<String>,
    pub level: Level,
    pub message: String,
}

/// 空白行返回 Ok(None)；未知格式返回 Err，调用者决定跳过还是严格失败。
/// 时间戳只接受 UTC 的 `YYYY-MM-DDTHH:MM:SSZ`，不会假装支持本地时区。
pub fn parse_line(line: &str) -> Result<Option<LogRecord>, &'static str> {
    let mut pieces = line.split_whitespace();
    let Some(first) = pieces.next() else {
        return Ok(None);
    };
    let (timestamp, raw_level) = if is_utc_timestamp(first) {
        (Some(first.to_owned()), pieces.next().ok_or("缺少日志级别")?)
    } else {
        (None, first)
    };
    let level = Level::parse(raw_level).ok_or("无法识别日志级别或时间戳")?;
    let message = pieces.collect::<Vec<_>>().join(" ");
    if message.is_empty() {
        return Err("缺少日志消息");
    }
    Ok(Some(LogRecord {
        timestamp,
        level,
        message,
    }))
}

/// 校验 UTC 格式和常见的时间字段边界（没有引入时区计算库）。
/// 无需使用的日期运算由今后的 chrono/serde 课程单独引入。
fn is_utc_timestamp(text: &str) -> bool {
    if text.len() != 20 || !text.is_ascii() {
        return false;
    }
    let bytes = text.as_bytes();
    if bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(i, b)| [4, 7, 10, 13, 16, 19].contains(&i) || b.is_ascii_digit())
    {
        return false;
    }
    let year = text[0..4].parse::<u16>().unwrap_or(0);
    let month = text[5..7].parse::<u8>().unwrap_or(0);
    let day = text[8..10].parse::<u8>().unwrap_or(0);
    let hours = text[11..13].parse::<u8>().unwrap_or(24);
    let minutes = text[14..16].parse::<u8>().unwrap_or(60);
    let seconds = text[17..19].parse::<u8>().unwrap_or(60);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    year > 0 && (1..=max_day).contains(&day) && hours < 24 && minutes < 60 && seconds < 60
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_two_useful_formats() {
        assert_eq!(
            parse_line("2026-10-28T12:30:00Z error cache miss").unwrap(),
            Some(LogRecord {
                timestamp: Some("2026-10-28T12:30:00Z".into()),
                level: Level::Error,
                message: "cache miss".into(),
            })
        );
        assert_eq!(
            parse_line("[WARN] slow   disk").unwrap().unwrap().message,
            "slow disk"
        );
        assert_eq!(
            parse_line("warning: deprecated API")
                .unwrap()
                .unwrap()
                .level,
            Level::Warn
        );
    }

    #[test]
    fn handles_blank_malformed_and_bad_timestamp() {
        assert_eq!(parse_line("   "), Ok(None));
        assert!(parse_line("ERROR").is_err());
        assert!(parse_line("not a log line").is_err());
        assert!(parse_line("2026-13-28T12:30:00Z INFO invalid date").is_err());
        assert!(parse_line("2026-10-28T99:30:00Z INFO invalid clock").is_err());
        assert!(parse_line("2025-02-29T12:00:00Z INFO invalid leap day").is_err());
        assert!(parse_line("2024-02-29T12:00:00Z INFO leap day").is_ok());
    }
}
