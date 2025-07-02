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

use anyhow::Result;
use chrono::NaiveDateTime;
use csv::ReaderBuilder;
use serde::Deserialize;

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
    /// Source: myClim R package (<https://github.com/ibot-geoecology/myClim>)
    /// References:
    /// - Wild et al. (2019), 10.1016/j.agrformet.2018.12.018 (soil types 1-9)
    /// - Kopecký et al. (2021), 10.1016/j.scitotenv.2020.143785 (universal)
    /// - Vlček (2010) Kalibrace vlhkostního čidla TST1 (TMS1 variants)
    fn coeffs(self) -> (f64, f64, f64) {
        match self {
            SoilType::Sand => (-3.00e-09, 0.000_161_192, -0.109_956_5),
            SoilType::LoamySandA => (-1.90e-08, 0.000_265_610, -0.154_089_3),
            SoilType::LoamySandB => (-2.30e-08, 0.000_282_473, -0.167_211_2),
            SoilType::SandyLoamA => (-3.80e-08, 0.000_339_449, -0.214_921_8),
            SoilType::SandyLoamB => (-9.00e-10, 0.000_261_847, -0.158_618_3),
            SoilType::Loam => (-5.10e-08, 0.000_397_984, -0.291_046_4),
            SoilType::SiltLoam => (1.70e-08, 0.000_118_119, -0.101_168_5),
            SoilType::Peat => (1.23e-07, -0.000_144_644, 0.202_927_9),
            SoilType::Water => (0.00e+00, 0.000_306_700, -0.134_927_9),
            SoilType::Universal => (-1.34e-08, 0.000_249_622, -0.157_888_8),
            SoilType::SandTms1 => (0.00e+00, 0.000_260_000, -0.133_040_0),
            SoilType::LoamySandTms1 => (0.00e+00, 0.000_330_000, -0.193_890_0),
            SoilType::SiltLoamTms1 => (0.00e+00, 0.000_380_000, -0.294_270_0),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            SoilType::Sand => "Sand",
            SoilType::LoamySandA => "Loamy Sand A",
            SoilType::LoamySandB => "Loamy Sand B",
            SoilType::SandyLoamA => "Sandy Loam A",
            SoilType::SandyLoamB => "Sandy Loam B",
            SoilType::Loam => "Loam",
            SoilType::SiltLoam => "Silt Loam",
            SoilType::Peat => "Peat",
            SoilType::Water => "Water",
            SoilType::Universal => "Universal",
            SoilType::SandTms1 => "Sand TMS1",
            SoilType::LoamySandTms1 => "Loamy Sand TMS1",
            SoilType::SiltLoamTms1 => "Silt Loam TMS1",
        }
    }

    pub const ALL: [SoilType; 13] = [
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
    ];
}

impl TryFrom<&str> for SoilType {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "sand" => Ok(Self::Sand),
            "loamy sand a" | "loamysanda" => Ok(Self::LoamySandA),
            "loamy sand b" | "loamysandb" => Ok(Self::LoamySandB),
            "sandy loam a" | "sandyloama" => Ok(Self::SandyLoamA),
            "sandy loam b" | "sandyloamb" => Ok(Self::SandyLoamB),
            "loam" => Ok(Self::Loam),
            "silt loam" | "siltloam" => Ok(Self::SiltLoam),
            "peat" => Ok(Self::Peat),
            "water" => Ok(Self::Water),
            "universal" => Ok(Self::Universal),
            "sand tms1" | "sandtms1" => Ok(Self::SandTms1),
            "loamy sand tms1" | "loamysandtms1" => Ok(Self::LoamySandTms1),
            "silt loam tms1" | "siltloamtms1" => Ok(Self::SiltLoamTms1),
            _ => Err(format!("Unknown soil type: {s}")),
        }
    }
}

// myClim temperature correction constants
// Source: myClim R package constants
const REF_T: f64 = 24.0; // Reference temperature (°C)
const ACOR_T: f64 = 1.911_327; // Temperature correction coefficient A
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
    vwc_cor.clamp(0.0, 1.0)
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
///
/// # Errors
///
/// This function returns an error if:
/// - The file at `path` cannot be opened or read
/// - CSV parsing fails due to invalid format
/// - `DateTime` parsing fails (expects format: "%Y.%m.%d %H:%M")
/// - Any field deserialization fails
pub fn process_file(path: String, soil: SoilType) -> Result<Vec<(NaiveDateTime, f64, f64, f64)>> {
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
