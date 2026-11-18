//! A private child module whose selected public types are reexported by its parent.

/// The activity on a study card; variants of a public enum are public too.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activity {
    Read,
    Practice,
}

/// A study card whose fields can only be set through the checked constructor.
#[derive(Debug)]
pub struct StudyCard {
    title: String,
    minutes: u32,
    activity: Activity,
}

impl StudyCard {
    /// Trim the title, reject a blank title, and validate the duration.
    pub fn new(title: &str, raw_minutes: &str, activity: Activity) -> Result<Self, &'static str> {
        let title = title.trim();
        if title.is_empty() {
            return Err("title must not be blank");
        }
        // `super::` refers to the parent language_basics module, not a file path.
        let minutes = super::parse_minutes(raw_minutes)?;
        Ok(Self {
            title: title.to_owned(),
            minutes,
            activity,
        })
    }

    /// Read the duration while keeping the field itself private.
    pub fn minutes(&self) -> u32 {
        self.minutes
    }

    /// Format a card using the public enum's two supported activities.
    pub fn summary(&self) -> String {
        let activity = match self.activity {
            Activity::Read => "read",
            Activity::Practice => "practice",
        };
        format!("{activity} | {} | {} min", self.title, self.minutes)
    }
}
