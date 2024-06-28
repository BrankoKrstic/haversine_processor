use serde::Serialize;

pub mod generate;
#[derive(Debug, Serialize)]
struct CoordPair {
    lat0: f64,
    lng0: f64,
    lat1: f64,
    lng1: f64,
}

impl From<((f64, f64), (f64, f64))> for CoordPair {
    fn from(value: ((f64, f64), (f64, f64))) -> Self {
        Self {
            lat0: value.0 .0,
            lng0: value.0 .1,
            lat1: value.1 .0,
            lng1: value.1 .1,
        }
    }
}
