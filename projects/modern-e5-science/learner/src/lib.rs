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

/// TODO(1): Welford population mean and standard deviation; validate inputs.
pub fn features(_samples: &[f64]) -> Result<Features, Error> {
    Err(Error::NotImplemented)
}

/// TODO(2): Fit mean and scale on training rows; protect zero variance.
pub fn fit_scaler(_rows: &[Features]) -> Result<Scaler, Error> {
    Err(Error::NotImplemented)
}

/// TODO(3): Train a bounded batch logistic model with stable loss and L2.
pub fn train(_rows: &[Example], _epochs: usize, _lr: f64) -> Result<Training, Error> {
    Err(Error::NotImplemented)
}

/// TODO(4): Count TP/TN/FP/FN at threshold 0.5, reject invalid probabilities.
pub fn evaluate(_rows: &[(bool, f64)]) -> Result<Confusion, Error> {
    Err(Error::NotImplemented)
}

pub mod synthetic;
