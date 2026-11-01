//! 纯确定性模拟：将一个真实、可编辑的配置映射成可以用 LogLens 分析的日志。
//! 数据是教学模拟，不代表生产环境请求或真实性能。
use std::fs;
use std::path::Path;

pub const LATENCIES: [u32; 12] = [120, 180, 210, 400, 520, 700, 230, 320, 450, 540, 680, 750];
pub const INCIDENT_TIMEOUT: u32 = 150;
pub const HEALTHY_TIMEOUT: u32 = 900;

pub fn parse_timeout(text: &str) -> Result<u32, String> {
    let mut timeout = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (key, value) = trimmed
            .split_once('=')
            .ok_or_else(|| format!("配置行缺少 '='：{trimmed}"))?;
        if key.trim() != "timeout_ms" {
            return Err(format!("不支持的配置项：{}", key.trim()));
        }
        if timeout.is_some() {
            return Err("重复配置 timeout_ms".into());
        }
        let parsed = value
            .trim()
            .parse::<u32>()
            .map_err(|_| "timeout_ms 必须为有效整数".to_string())?;
        if !(1..=5000).contains(&parsed) {
            return Err("timeout_ms 必须在 1..=5000 范围内".into());
        }
        timeout = Some(parsed);
    }
    timeout.ok_or_else(|| "缺少 timeout_ms 配置".into())
}

pub fn read_timeout(project: &Path) -> Result<u32, String> {
    let path = project.join("config/service.conf");
    let text =
        fs::read_to_string(&path).map_err(|err| format!("读取 {} 失败：{err}", path.display()))?;
    parse_timeout(&text)
}

pub fn render_log(timeout: u32) -> String {
    let mut output = String::new();
    for (i, latency) in LATENCIES.iter().enumerate() {
        let level = if *latency > timeout { "ERROR" } else { "INFO" };
        output.push_str(&format!(
            "2026-10-28T09:{:02}:00Z {level} gateway request={} latency_ms={latency} timeout_ms={timeout}\n",
            i, i + 1
        ));
    }
    output
}

/// 生成的独立 Rust 微应用源码。学员能自己用 Cargo 运行并检查产生的日志。
/// 它与 LabQuest 中的确定性模拟规则保持一致，并有真实编译回归测试。
pub const MINI_APP: &str = r#"use std::fs;
use std::path::PathBuf;

const LATENCIES: [u32; 12] = [120, 180, 210, 400, 520, 700, 230, 320, 450, 540, 680, 750];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let conf = fs::read_to_string(root.join("config/service.conf"))?;
    let (_, value) = conf.lines()
        .filter(|line| !line.trim().starts_with('#') && !line.trim().is_empty())
        .find_map(|line| line.split_once('='))
        .ok_or("缺少 timeout_ms 配置")?;
    // 默认读取实际服务配置；命令行覆盖仅用于练习程序的多场景验证。
    // 不修改配置文件，学员可以独立测试同一份编译结果对不同输入的反应。
    let args: Vec<String> = std::env::args().skip(1).collect();
    let timeout: u32 = match args.as_slice() {
        [] => value.trim().parse()?,
        [flag, amount] if flag == "--timeout-ms" => amount.parse()?,
        _ => return Err("用法：case-gateway [--timeout-ms <整数>]".into()),
    };
    if !(1..=5000).contains(&timeout) {
        return Err("timeout_ms 必须在 1..=5000 范围内".into());
    }
    for (index, latency) in LATENCIES.iter().enumerate() {
        let level = if *latency > timeout { "ERROR" } else { "INFO" };
        println!(
            "2026-10-28T09:{:02}:00Z {level} gateway request={} latency_ms={latency} timeout_ms={timeout}",
            index, index + 1
        );
    }
    // TODO(CASE-01): 统计错误次数，打印精确的一行：
    // SUMMARY requests=12 errors=N
    // 其中 N 要由 LATENCIES 和 timeout 计算，不要写死数字。
    Ok(())
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_log_matches_expected_error_counts() {
        let incident = render_log(INCIDENT_TIMEOUT);
        assert_eq!(incident.lines().count(), 12);
        assert_eq!(incident.matches(" ERROR ").count(), 11);
        assert_eq!(render_log(HEALTHY_TIMEOUT).matches(" ERROR ").count(), 0);
    }

    #[test]
    fn parser_rejects_bad_configs() {
        for bad in [
            "",
            "timeout_ms=0",
            "timeout_ms=5001",
            "timeout_ms=xx",
            "other=200",
            "timeout_ms=200\ntimeout_ms=300",
        ] {
            assert!(parse_timeout(bad).is_err(), "{bad:?}");
        }
        assert_eq!(parse_timeout("# hi\ntimeout_ms = 800\n"), Ok(800));
    }
}
