//! CPU-only teaching experiment on deterministic synthetic signals.
pub const MAX_ITEMS: usize = 4096;
const LIMIT: f64 = 1_000_000.0;

pub type Features = [f64; 2];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Length,
    NonFiniteOrRange,
    Hyperparameter,
    OneClass,
    NotImplemented,
}

fn bounded(x: f64) -> bool {
    x.is_finite() && x.abs() <= LIMIT
}

#[derive(Clone, Copy, Debug)]
pub struct Example {
    pub x: Features,
    pub label: bool,
}

#[derive(Clone, Debug)]
pub struct Scaler {
    mean: Features,
    scale: Features,
}

impl Scaler {
    pub fn transform(&self, x: Features) -> Result<Features, Error> {
        if !x.iter().all(|&value| bounded(value)) {
            return Err(Error::NonFiniteOrRange);
        }
        Ok([(x[0] - self.mean[0]) / self.scale[0],
            (x[1] - self.mean[1]) / self.scale[1]])
    }
}

fn sigmoid(z: f64) -> f64 {
    if z >= 0.0 {
        1.0 / (1.0 + (-z).exp())
    } else {
        let e = z.exp();
        e / (1.0 + e)
    }
}

#[derive(Clone, Debug)]
pub struct Model {
    scaler: Scaler,
    weights: Features,
    bias: f64,
}

impl Model {
    pub fn probability(&self, x: Features) -> Result<f64, Error> {
        let v = self.scaler.transform(x)?;
        Ok(sigmoid(self.bias + self.weights[0] * v[0] + self.weights[1] * v[1]))
    }
}

#[derive(Clone, Debug)]
pub struct Training {
    pub model: Model,
    pub loss_before: f64,
    pub loss_after: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Confusion {
    pub tp: u32,
    pub tn: u32,
    pub fp: u32,
    pub fn_: u32,
}

impl Confusion {
    pub fn total(&self) -> u32 {
        self.tp + self.tn + self.fp + self.fn_
    }
    pub fn accuracy(&self) -> Option<f64> {
        (self.total() > 0).then(|| (self.tp + self.tn) as f64 / self.total() as f64)
    }
}

/// Welford population mean and standard deviation of a bounded signal window.
pub fn features(samples: &[f64]) -> Result<Features, Error> {
    if !(2..=MAX_ITEMS).contains(&samples.len()) {
        return Err(Error::Length);
    }
    let mut mean = 0.0;
    let mut m2 = 0.0;
    for (i, &x) in samples.iter().enumerate() {
        if !bounded(x) {
            return Err(Error::NonFiniteOrRange);
        }
        let delta = x - mean;
        mean += delta / (i + 1) as f64;
        m2 += delta * (x - mean);
    }
    Ok([mean, (m2 / samples.len() as f64).max(0.0).sqrt()])
}

/// Fit ONLY on training examples. Constant dimensions use scale 1.
pub fn fit_scaler(rows: &[Features]) -> Result<Scaler, Error> {
    if !(2..=MAX_ITEMS).contains(&rows.len()) {
        return Err(Error::Length);
    }
    let mut mean = [0.0; 2];
    let mut m2 = [0.0; 2];
    for (i, row) in rows.iter().enumerate() {
        for j in 0..2 {
            if !bounded(row[j]) {
                return Err(Error::NonFiniteOrRange);
            }
            let delta = row[j] - mean[j];
            mean[j] += delta / (i + 1) as f64;
            m2[j] += delta * (row[j] - mean[j]);
        }
    }
    let mut scale = [1.0; 2];
    for j in 0..2 {
        let sd = (m2[j] / rows.len() as f64).max(0.0).sqrt();
        if sd >= 1e-9 {
            scale[j] = sd;
        }
    }
    Ok(Scaler { mean, scale })
}

/// Batch logistic regression on standardized training data, fixed L2=0.01.
/// Returns unregularized binary cross-entropy before and after fitting.
pub fn train(rows: &[Example], epochs: usize, lr: f64) -> Result<Training, Error> {
    if !(2..=MAX_ITEMS).contains(&rows.len()) {
        return Err(Error::Length);
    }
    if !(1..=1000).contains(&epochs) || !lr.is_finite() || lr <= 0.0 || lr > 0.2 {
        return Err(Error::Hyperparameter);
    }
    if !rows.iter().any(|row| row.label) || !rows.iter().any(|row| !row.label) {
        return Err(Error::OneClass);
    }
    let raw: Vec<_> = rows.iter().map(|row| row.x).collect();
    let scaler = fit_scaler(&raw)?;
    let x: Vec<_> = raw.iter().map(|&v| scaler.transform(v)).collect::<Result<_, _>>()?;
    let n = rows.len() as f64;
    let mut weights = [0.0; 2];
    let mut bias = 0.0;
    let cross_entropy = |w: Features, b: f64| {
        x.iter().zip(rows).map(|(v, row)| {
            let z = b + w[0] * v[0] + w[1] * v[1];
            let y = if row.label { 1.0 } else { 0.0 };
            z.max(0.0) - y * z + (-z.abs()).exp().ln_1p()
        }).sum::<f64>() / n
    };
    let loss_before = cross_entropy(weights, bias);
    for _ in 0..epochs {
        let mut grad = [0.0; 2];
        let mut gb = 0.0;
        for (v, row) in x.iter().zip(rows) {
            let z = bias + weights[0] * v[0] + weights[1] * v[1];
            let y = if row.label { 1.0 } else { 0.0 };
            let error = sigmoid(z) - y;
            for j in 0..2 {
                grad[j] += error * v[j];
            }
            gb += error;
        }
        for j in 0..2 {
            weights[j] -= lr * (grad[j] / n + 0.01 * weights[j]);
        }
        bias -= lr * gb / n;
    }
    let loss_after = cross_entropy(weights, bias);
    Ok(Training { model: Model { scaler, weights, bias }, loss_before, loss_after })
}

/// Compute metrics from held-out labels and probabilities, threshold 0.5.
pub fn evaluate(rows: &[(bool, f64)]) -> Result<Confusion, Error> {
    if rows.is_empty() || rows.len() > MAX_ITEMS {
        return Err(Error::Length);
    }
    let mut result = Confusion::default();
    for &(label, p) in rows {
        if !p.is_finite() || !(0.0..=1.0).contains(&p) {
            return Err(Error::NonFiniteOrRange);
        }
        match (label, p >= 0.5) {
            (true, true) => result.tp += 1,
            (false, false) => result.tn += 1,
            (false, true) => result.fp += 1,
            (true, false) => result.fn_ += 1,
        }
    }
    Ok(result)
}

pub mod synthetic;
