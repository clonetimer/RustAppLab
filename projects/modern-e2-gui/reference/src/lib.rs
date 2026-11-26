//! SignalDesk: bounded local numeric CSV, explicit state transitions and native GUI.
//! This is a teaching viewer, not a general CSV dialect or safety alarm system.
use std::{fs::{File, OpenOptions}, io::{Read, Write}, path::Path};

pub const MAX_BYTES: usize = 256 * 1024;
pub const MAX_SAMPLES: usize = 4096;
pub const DEMO: &str = "time_s,value\n0,21\n1,24\n2,28\n3,33\n4,35\n5,29\n6,31\n7,26\n";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample { pub time_s: u32, pub value: f64 }

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats { pub count: usize, pub mean: f64, pub min: f64, pub max: f64, pub alerts: usize }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error { Header, Row, Empty, Limit, Order, Range, File, Todo }

pub fn validate(samples: &[Sample]) -> Result<(), Error> {
    if samples.is_empty() { return Err(Error::Empty); }
    if samples.len() > MAX_SAMPLES { return Err(Error::Limit); }
    if samples.iter().any(|s| !s.value.is_finite() || !(-1000.0..=1000.0).contains(&s.value)) {
        return Err(Error::Range);
    }
    if samples.windows(2).any(|p| p[0].time_s >= p[1].time_s) { return Err(Error::Order); }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language { English, Chinese }

impl Language {
    pub fn text(self, en: &'static str, zh: &'static str) -> &'static str {
        match self { Self::English => en, Self::Chinese => zh }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Replace(Vec<Sample>), Threshold(f64), OnlyAlerts(bool), ResetDemo, ToggleLanguage
}

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub samples: Vec<Sample>,
    pub threshold: f64,
    pub only_alerts: bool,
    pub language: Language,
}

impl Model {
    /// The seed is fixed infrastructure so a learner can test each step independently.
    pub fn demo(language: Language) -> Self {
        let samples = [21.0,24.0,28.0,33.0,35.0,29.0,31.0,26.0].into_iter()
            .enumerate().map(|(i,value)| Sample { time_s:i as u32, value }).collect();
        Self { samples, threshold:30.0, only_alerts:false, language }
    }

    pub fn visible(&self) -> Vec<Sample> {
        self.samples.iter().copied().filter(|s| !self.only_alerts || s.value >= self.threshold).collect()
    }
}

pub fn parse_samples(text: &str) -> Result<Vec<Sample>, Error> {
    if text.len() > MAX_BYTES { return Err(Error::Limit); }
    let mut lines = text.lines();
    if lines.next() != Some("time_s,value") { return Err(Error::Header); }
    let mut samples = Vec::new();
    for line in lines {
        if samples.len() == MAX_SAMPLES { return Err(Error::Limit); }
        let (time, value) = line.split_once(',').ok_or(Error::Row)?;
        let time_s = time.trim().parse::<u32>().map_err(|_| Error::Row)?;
        let value = value.trim().parse::<f64>().map_err(|_| Error::Row)?;
        samples.push(Sample { time_s, value });
    }
    validate(&samples)?;
    Ok(samples)
}

pub fn summarize(samples: &[Sample], threshold: f64) -> Option<Stats> {
    if validate(samples).is_err() || !(-1000.0..=1000.0).contains(&threshold) { return None; }
    let mut min = samples[0].value;
    let mut max = min;
    let mut sum = 0.0;
    let mut alerts = 0;
    for row in samples {
        min = min.min(row.value);
        max = max.max(row.value);
        sum += row.value;
        if row.value >= threshold { alerts += 1; }
    }
    Some(Stats { count:samples.len(), mean:sum/samples.len() as f64, min, max, alerts })
}

impl Model {
    /// Invalid input leaves every field of the current state unchanged.
    pub fn apply(&mut self, action: Action) -> Result<(), Error> {
        match action {
            Action::Replace(samples) => { validate(&samples)?; self.samples = samples; }
            Action::Threshold(value) => {
                if !(-1000.0..=1000.0).contains(&value) { return Err(Error::Range); }
                self.threshold = value;
            }
            Action::OnlyAlerts(value) => self.only_alerts = value,
            Action::ResetDemo => *self = Self::demo(self.language),
            Action::ToggleLanguage => self.language = match self.language {
                Language::English => Language::Chinese, Language::Chinese => Language::English
            },
        }
        Ok(())
    }
}

pub fn read_csv(path: &Path) -> Result<Vec<Sample>, Error> {
    let mut bytes = Vec::new();
    File::open(path).map_err(|_| Error::File)?.take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes).map_err(|_| Error::File)?;
    if bytes.len() > MAX_BYTES { return Err(Error::Limit); }
    let text = std::str::from_utf8(&bytes).map_err(|_| Error::Row)?;
    parse_samples(text)
}

/// Export only the currently visible data. Never overwrite an existing path.
/// A failed write may leave a partial newly created file; report the error.
pub fn export_csv(path: &Path, model: &Model) -> Result<(), Error> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path).map_err(|_| Error::File)?;
    file.write_all(b"time_s,value\n").map_err(|_| Error::File)?;
    for row in model.visible() {
        writeln!(file, "{},{}", row.time_s, row.value).map_err(|_| Error::File)?;
    }
    file.sync_all().map_err(|_| Error::File)
}
pub mod gui;
