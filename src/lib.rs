/*
 * VWC (Volumetric Water Content) Calculation Library
 *
 * This implementation is based on the myClim R package algorithms and coefficients.
 * Original myClim package: https://github.com/ibot-geoecology/myClim
 *
 * Copyright notice for myClim-derived components:
 * The VWC calculation algorithm, soil type coefficients, and temperature correction
 * constants are derived from the myClim R package, which is licensed under GPL v2.
 *
 * myClim package authors and contributors:
 * - Institute of Botany of the Czech Academy of Sciences
 * - See: https://github.com/ibot-geoecology/myClim
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

use chrono::NaiveDateTime;
use csv::ReaderBuilder;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SoilType {
    Sand,
    LoamySandA,
    LoamySandB,
    SandyLoamA,
    SandyLoamB,
    Loam,
    SiltLoam,
    Peat,
    Water,
    Universal,
    SandTms1,
    LoamySandTms1,
    SiltLoamTms1,
}

impl SoilType {
    /// Soil type coefficients (a, b, c) for VWC = a·count² + b·count + c
    /// Source: myClim R package (https://github.com/ibot-geoecology/myClim)
    /// References:
    /// - Wild et al. (2019), 10.1016/j.agrformet.2018.12.018 (soil types 1-9)
    /// - Kopecký et al. (2021), 10.1016/j.scitotenv.2020.143785 (universal)
    /// - Vlček (2010) Kalibrace vlhkostního čidla TST1 (TMS1 variants)
    fn coeffs(&self) -> (f64, f64, f64) {
        match self {
            SoilType::Sand => (-3.00e-09, 0.000161192, -0.1099565),
            SoilType::LoamySandA => (-1.90e-08, 0.000265610, -0.1540893),
            SoilType::LoamySandB => (-2.30e-08, 0.000282473, -0.1672112),
            SoilType::SandyLoamA => (-3.80e-08, 0.000339449, -0.2149218),
            SoilType::SandyLoamB => (-9.00e-10, 0.000261847, -0.1586183),
            SoilType::Loam => (-5.10e-08, 0.000397984, -0.2910464),
            SoilType::SiltLoam => (1.70e-08, 0.000118119, -0.1011685),
            SoilType::Peat => (1.23e-07, -0.000144644, 0.2029279),
            SoilType::Water => (0.00e+00, 0.000306700, -0.1349279),
            SoilType::Universal => (-1.34e-08, 0.000249622, -0.1578888),
            SoilType::SandTms1 => (0.00e+00, 0.000260000, -0.1330400),
            SoilType::LoamySandTms1 => (0.00e+00, 0.000330000, -0.1938900),
            SoilType::SiltLoamTms1 => (0.00e+00, 0.000380000, -0.2942700),
        }
    }

    /// Get soil type from string name
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "sand" => Ok(SoilType::Sand),
            "loamy sand a" | "loamysanda" => Ok(SoilType::LoamySandA),
            "loamy sand b" | "loamysandb" => Ok(SoilType::LoamySandB),
            "sandy loam a" | "sandyloama" => Ok(SoilType::SandyLoamA),
            "sandy loam b" | "sandyloamb" => Ok(SoilType::SandyLoamB),
            "loam" => Ok(SoilType::Loam),
            "silt loam" | "siltloam" => Ok(SoilType::SiltLoam),
            "peat" => Ok(SoilType::Peat),
            "water" => Ok(SoilType::Water),
            "universal" => Ok(SoilType::Universal),
            "sand tms1" | "sandtms1" => Ok(SoilType::SandTms1),
            "loamy sand tms1" | "loamysandtms1" => Ok(SoilType::LoamySandTms1),
            "silt loam tms1" | "siltloamtms1" => Ok(SoilType::SiltLoamTms1),
            _ => Err(format!("Unknown soil type: {}", s)),
        }
    }

    /// Get string representation
    pub fn to_str(&self) -> &'static str {
        match self {
            SoilType::Sand => "sand",
            SoilType::LoamySandA => "loamy sand A",
            SoilType::LoamySandB => "loamy sand B",
            SoilType::SandyLoamA => "sandy loam A",
            SoilType::SandyLoamB => "sandy loam B",
            SoilType::Loam => "loam",
            SoilType::SiltLoam => "silt loam",
            SoilType::Peat => "peat",
            SoilType::Water => "water",
            SoilType::Universal => "universal",
            SoilType::SandTms1 => "sand TMS1",
            SoilType::LoamySandTms1 => "loamy sand TMS1",
            SoilType::SiltLoamTms1 => "silt loam TMS1",
        }
    }

    /// List all available soil types
    pub fn all_types() -> Vec<SoilType> {
        vec![
            SoilType::Sand,
            SoilType::LoamySandA,
            SoilType::LoamySandB,
            SoilType::SandyLoamA,
            SoilType::SandyLoamB,
            SoilType::Loam,
            SoilType::SiltLoam,
            SoilType::Peat,
            SoilType::Water,
            SoilType::Universal,
            SoilType::SandTms1,
            SoilType::LoamySandTms1,
            SoilType::SiltLoamTms1,
        ]
    }
}

// myClim temperature correction constants
// Source: myClim R package constants
const REF_T: f64 = 24.0; // Reference temperature (°C)
const ACOR_T: f64 = 1.911327; // Temperature correction coefficient A
const WCOR_T: f64 = 0.64108; // Temperature correction coefficient W

/// Calculate VWC using the myClim algorithm
///
/// This function implements the exact algorithm from the myClim R package:
/// 1. Calculate initial VWC from raw sensor values
/// 2. Apply temperature correction to raw values  
/// 3. Recalculate VWC with temperature-corrected values
/// 4. Apply calibration corrections (if any)
/// 5. Clamp result between 0 and 1
///
/// # Arguments
/// * `raw_value` - Raw moisture sensor reading
/// * `temp_value` - Temperature reading (°C)
/// * `soil` - Soil type for coefficient selection
///
/// # Returns
/// Volumetric Water Content (VWC) as a fraction (0.0 to 1.0)
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
