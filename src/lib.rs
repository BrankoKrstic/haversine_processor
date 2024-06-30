use serde::{Deserialize, Serialize};

pub mod calc;
pub mod generate;
pub mod metrics;
pub mod parser;

#[derive(Debug, Serialize, Deserialize)]
pub struct CoordPair {
    lat0: f64,
    lon0: f64,
    lat1: f64,
    lon1: f64,
}

impl From<((f64, f64), (f64, f64))> for CoordPair {
    fn from(value: ((f64, f64), (f64, f64))) -> Self {
        Self {
            lat0: value.0 .0,
            lon0: value.0 .1,
            lat1: value.1 .0,
            lon1: value.1 .1,
        }
    }
}
