//! Fixed synthetic event groups. No real spacecraft telemetry.
use crate::{Error, Example, features};

pub struct Window {
    pub group: u32,
    pub label: bool,
    pub samples: Vec<f64>,
}

pub fn grouped_windows() -> (Vec<Window>, Vec<Window>) {
    // Group selection precedes extraction. Test IDs never occur in training.
    fn generate(first: u32, count: u32) -> Vec<Window> {
        let mut windows = Vec::new();
        for group in first..first + count {
            let label = group % 2 == 1;
            for window in 0..3 {
                let offset = if label { 2.5 } else { 0.5 };
                let amplitude = if label { 0.9 } else { 0.2 };
                let shift = (group % 7) as f64 * 0.035 + window as f64 * 0.025;
                let samples = (0..64).map(|i| {
                    // Deterministic but varying finite signal; not a physical model.
                    let phase = (i * 17 + group as usize * 11 + window * 7) % 31;
                    offset + shift + amplitude * (phase as f64 / 15.0 - 1.0)
                }).collect();
                windows.push(Window { group, label, samples });
            }
        }
        windows
    }
    (generate(0, 16), generate(100, 8))
}

pub fn extract(windows: &[Window]) -> Result<Vec<Example>, Error> {
    windows.iter().map(|window| {
        Ok(Example { x: features(&window.samples)?, label: window.label })
    }).collect()
}
