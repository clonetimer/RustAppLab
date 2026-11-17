//! A deliberately small, dependency-free library for Rust Book chapter 14.
//!
//! The data lives in a library; the Cargo extension binary is only an adapter.
//!
//! ```
//! use rustapplab_book_card::describe;
//! assert!(describe("workspace").unwrap().starts_with("14.3"));
//! ```

/// Exactly the five subsections of the 2018-edition Rust Book chapter 14.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topic {
    Profiles,
    Publish,
    Workspace,
    Install,
    Extension,
}

impl Topic {
    pub fn parse(input: &str) -> Option<Self> {
        match input {
            "profiles" => Some(Self::Profiles),
            "publish" => Some(Self::Publish),
            "workspace" => Some(Self::Workspace),
            "install" => Some(Self::Install),
            "extension" => Some(Self::Extension),
            _ => None,
        }
    }

    pub fn chapter(self) -> &'static str {
        match self {
            Self::Profiles => "14.1",
            Self::Publish => "14.2",
            Self::Workspace => "14.3",
            Self::Install => "14.4",
            Self::Extension => "14.5",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Profiles => "Workspace 根 Cargo.toml 中的 dev/release profile",
            Self::Publish => "先检查 crate 打包清单；练习不上传公共仓库",
            Self::Workspace => "共享 Cargo.lock、多个 package 与 path 依赖",
            Self::Install => "cargo install --path --root 安装到专用练习目录",
            Self::Extension => "cargo-<name> 可执行程序被 Cargo 发现为子命令",
        }
    }
}

pub fn describe(input: &str) -> Result<String, &'static str> {
    let topic = Topic::parse(input).ok_or("未知主题；可选 profiles/publish/workspace/install/extension")?;
    Ok(format!("{} {}\n", topic.chapter(), topic.description()))
}

#[cfg(test)]
mod tests {
    use super::{describe, Topic};

    #[test]
    fn each_book_subsection_has_unique_chapter() {
        let topics = [
            Topic::Profiles,
            Topic::Publish,
            Topic::Workspace,
            Topic::Install,
            Topic::Extension,
        ];
        let actual: Vec<_> = topics.iter().map(|topic| topic.chapter()).collect();
        assert_eq!(actual, ["14.1", "14.2", "14.3", "14.4", "14.5"]);
    }

    #[test]
    fn every_topic_explains_a_real_operation() {
        for key in ["profiles", "publish", "workspace", "install", "extension"] {
            let explanation = describe(key).unwrap();
            assert!(explanation.starts_with("14."));
            assert!(explanation.ends_with('\n'));
        }
    }

    #[test]
    fn unknown_topics_are_rejected_not_silently_mapped() {
        assert!(Topic::parse("unsafe").is_none());
        assert!(describe("Profile").is_err());
        assert!(describe("").is_err());
    }

    #[test]
    fn descriptions_match_the_expected_subchapters() {
        assert!(describe("profiles").unwrap().starts_with("14.1 "));
        assert!(describe("publish").unwrap().starts_with("14.2 "));
        assert!(describe("workspace").unwrap().starts_with("14.3 "));
        assert!(describe("install").unwrap().starts_with("14.4 "));
        assert!(describe("extension").unwrap().starts_with("14.5 "));
    }
}
