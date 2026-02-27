/*
 * Gas Flux Calculation Library
 *
 * This implementation mirrors the MATLAB script reader_gas_Flux_EPFL_may.m
 * for computing gas fluxes from LI-7810 chamber measurements.
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

#![allow(
    clippy::similar_names,
    clippy::cast_precision_loss,
    clippy::too_many_arguments
)]

use serde::{Deserialize, Serialize};

/// Universal gas constant [J/(mol·K)]
const R_GAS: f64 = 8.314;

/// Result of a gas flux calculation from chamber measurement time series.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasFluxResult {
    /// CO₂ flux [μmol m⁻² s⁻¹]
    pub flux_co2_umol_m2_s: f64,
    /// CH₄ flux [nmol m⁻² s⁻¹]
    pub flux_ch4_nmol_m2_s: f64,
    /// H₂O flux [μmol m⁻² s⁻¹]
    pub flux_h2o_umol_m2_s: f64,
    /// R² of CO₂ linear fit
    pub r2_co2: f64,
    /// R² of CH₄ linear fit
    pub r2_ch4: f64,
    /// R² of H₂O linear fit
    pub r2_h2o: f64,
}

/// Simple linear regression: returns (slope, r²).
///
/// Uses the ordinary least-squares formula:
///   slope = Σ((x-x̄)(y-ȳ)) / Σ((x-x̄)²)
///   r     = Σ((x-x̄)(y-ȳ)) / sqrt(Σ((x-x̄)²) · Σ((y-ȳ)²))
///   r²    = r * r
fn linear_regression(x: &[f64], y: &[f64]) -> (f64, f64) {
    let n = x.len() as f64;
    let x_mean = x.iter().sum::<f64>() / n;
    let y_mean = y.iter().sum::<f64>() / n;

    let mut ss_xy = 0.0;
    let mut ss_xx = 0.0;
    let mut ss_yy = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - x_mean;
        let dy = y[i] - y_mean;
        ss_xy += dx * dy;
        ss_xx += dx * dx;
        ss_yy += dy * dy;
    }

    let slope = if ss_xx.abs() < f64::EPSILON {
        0.0
    } else {
        ss_xy / ss_xx
    };

    let r2 = if ss_xx.abs() < f64::EPSILON || ss_yy.abs() < f64::EPSILON {
        0.0
    } else {
        let r = ss_xy / (ss_xx * ss_yy).sqrt();
        r * r
    };

    (slope, r2)
}

/// Compute gas fluxes from chamber measurement time series.
///
/// This implements the same algorithm as the MATLAB script
/// `reader_gas_Flux_EPFL_may.m`:
///
/// - CO₂: linear regression of (ppm→mol/mol) vs time, then
///   `flux = slope × (P/(R·T)) × (V/A) × 10⁶` → μmol m⁻² s⁻¹
/// - CH₄: linear regression of ppb vs time, slope×10⁻⁹ for mol/mol/s,
///   then `flux = slope × (P/(R·T)) × (V/A) × 10⁹` → nmol m⁻² s⁻¹
/// - H₂O: linear regression of mmol/mol vs time, slope×10⁻³,
///   then `flux = slope × (P/(R·T)) × (V/A) × 10⁶` → μmol m⁻² s⁻¹
///
/// # Arguments
///
/// * `timestamps_s` - Elapsed time in seconds from measurement start
/// * `co2_ppm` - CO₂ concentration [ppm]
/// * `ch4_ppb` - CH₄ concentration [ppb]
/// * `h2o_mmol_mol` - H₂O concentration [mmol mol⁻¹]
/// * `chamber_temp_c` - Chamber temperature [°C]
/// * `chamber_pressure_kpa` - Chamber pressure [kPa]
/// * `total_volume_m3` - Total system volume [m³]
/// * `chamber_area_m2` - Chamber area [m²]
///
/// # Panics
///
/// Panics if any input slice is empty.
#[must_use]
pub fn compute_gas_flux(
    timestamps_s: &[f64],
    co2_ppm: &[f64],
    ch4_ppb: &[f64],
    h2o_mmol_mol: &[f64],
    chamber_temp_c: &[f64],
    chamber_pressure_kpa: &[f64],
    total_volume_m3: f64,
    chamber_area_m2: f64,
) -> GasFluxResult {
    assert!(!timestamps_s.is_empty(), "timestamps must not be empty");

    // Mean temperature [K] and pressure [Pa]
    let t_k = chamber_temp_c.iter().sum::<f64>() / chamber_temp_c.len() as f64 + 273.15;
    let p_pa =
        chamber_pressure_kpa.iter().sum::<f64>() / chamber_pressure_kpa.len() as f64 * 1000.0;

    let pv_art = (p_pa / (R_GAS * t_k)) * (total_volume_m3 / chamber_area_m2);

    // CO2: convert ppm to mol/mol, then linear regression
    let co2_mol: Vec<f64> = co2_ppm.iter().map(|&v| v * 1e-6).collect();
    let (slope_co2, r2_co2) = linear_regression(timestamps_s, &co2_mol);
    let flux_co2 = slope_co2 * pv_art * 1e6;

    // CH4: linear regression on raw ppb, then convert
    let (slope_ch4_raw, r2_ch4) = linear_regression(timestamps_s, ch4_ppb);
    let flux_ch4 = slope_ch4_raw * 1e-9 * pv_art * 1e9;

    // H2O: linear regression on raw mmol/mol, then convert
    let (slope_h2o_raw, r2_h2o) = linear_regression(timestamps_s, h2o_mmol_mol);
    let flux_h2o = slope_h2o_raw * 1e-3 * pv_art * 1e6;

    GasFluxResult {
        flux_co2_umol_m2_s: flux_co2,
        flux_ch4_nmol_m2_s: flux_ch4,
        flux_h2o_umol_m2_s: flux_h2o,
        r2_co2,
        r2_ch4,
        r2_h2o,
    }
}

// ---------------------------------------------------------------------------
// Unit tests for private linear_regression function
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_regression_perfect_line() {
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y = vec![1.0, 3.0, 5.0, 7.0, 9.0]; // y = 2x + 1
        let (slope, r2) = linear_regression(&x, &y);
        assert!(
            (slope - 2.0).abs() < 1e-10,
            "slope should be 2.0, got {slope}"
        );
        assert!((r2 - 1.0).abs() < 1e-10, "r2 should be 1.0, got {r2}");
    }

    #[test]
    fn test_linear_regression_noisy() {
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y = vec![1.0, 2.5, 5.5, 6.5, 9.0];
        let (slope, r2) = linear_regression(&x, &y);
        assert!((slope - 2.0).abs() < 0.1, "slope ~2.0, got {slope}");
        assert!(r2 > 0.95, "r2 should be high, got {r2}");
    }
}
