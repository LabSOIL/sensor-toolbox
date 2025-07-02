use chrono::NaiveDateTime;
use csv::ReaderBuilder;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Clone, Copy)]
pub enum SoilType {
    Peat,
    Universal,
}

impl SoilType {
    /// (a, b, c) for VWC = a·count² + b·count + c
    fn coeffs(&self) -> (f64, f64, f64) {
        match self {
            SoilType::Peat => (1.23e-07, -0.000144644, 0.2029279),
            SoilType::Universal => (-1.34e-08, 0.000249622, -0.1578888),
        }
    }
}

fn mc_calc_vwc(count: f64, soil: SoilType) -> f64 {
    let (a, b, c) = soil.coeffs();
    a * count * count + b * count + c
}

#[derive(Debug, Deserialize)]
struct RawRecord {
    #[serde(rename = "1")]
    datetime: String,
    #[serde(rename = "6")]
    raw: f64,
    #[serde(rename = "7")]
    temp: f64,
}

/// Read `<path>`, compute VWC for `soil`, return (datetime, raw, temp, vwc).
pub fn process_file(
    path: &str,
    soil: SoilType,
) -> Result<Vec<(NaiveDateTime, f64, f64, f64)>, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_path(path)?;
    let mut out = Vec::new();
    for result in rdr.deserialize() {
        let rec: RawRecord = result?;
        let dt = NaiveDateTime::parse_from_str(&rec.datetime, "%Y.%m.%d %H:%M")?;
        let vwc = mc_calc_vwc(rec.raw, soil);
        out.push((dt, rec.raw, rec.temp, vwc));
    }
    Ok(out)
}
