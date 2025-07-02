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
    /// Updated to match exact myClim coefficients
    fn coeffs(&self) -> (f64, f64, f64) {
        match self {
            SoilType::Peat => (1.23e-07, -0.000144644, 0.2029279),
            SoilType::Universal => (-1.34e-08, 0.000249622, -0.1578888),
        }
    }
}

// myClim temperature correction constants
const REF_T: f64 = 24.0;
const ACOR_T: f64 = 1.911327;
const WCOR_T: f64 = 0.64108;

fn mc_calc_vwc(raw_value: f64, temp_value: f64, soil: SoilType) -> f64 {
    let (a, b, c) = soil.coeffs();

    // Step 1: Initial VWC calculation
    let vwc = a * raw_value * raw_value + b * raw_value + c;

    // Step 2: Temperature correction (from myClim source)
    let dcor_t = WCOR_T - ACOR_T;
    let tcor = if temp_value.is_nan() {
        raw_value
    } else {
        raw_value + (REF_T - temp_value) * (ACOR_T + dcor_t * vwc)
    };

    // Step 3: Temperature-corrected VWC calculation
    // Note: cal_cor_factor and cal_cor_slope are 0 for uncalibrated data
    let cal_cor_factor = 0.0;
    let cal_cor_slope = 0.0;
    let corrected_raw = tcor + cal_cor_factor + cal_cor_slope * vwc;
    let vwc_cor = a * corrected_raw * corrected_raw + b * corrected_raw + c;

    // Step 4: Clamp result between 0 and 1 (pmin(pmax(vwc_cor, 0), 1))
    vwc_cor.max(0.0).min(1.0)
}

#[derive(Debug, Deserialize)]
struct RawRecord {
    _field0: String,  // index 0
    datetime: String, // index 1
    _field2: String,  // index 2
    temp: f64,        // index 3 - temperature field
    _field4: String,  // index 4
    _field5: String,  // index 5
    raw: f64,         // index 6 - raw count for VWC calculation
    _field7: String,  // index 7
    _field8: String,  // index 8
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
        let vwc = mc_calc_vwc(rec.raw, rec.temp, soil);
        out.push((dt, rec.raw, rec.temp, vwc));
    }
    Ok(out)
}
