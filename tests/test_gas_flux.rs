#![allow(clippy::cast_precision_loss)]

use serde::Deserialize;
use soil_sensor_toolbox::compute_gas_flux;

// ---------------------------------------------------------------------------
// Fixture data structures for loading Balmoos JSON test data
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct FixtureData {
    #[allow(dead_code)]
    collar: String,
    #[allow(dead_code)]
    replicate: String,
    total_volume_ml: f64,
    area_cm2: f64,
    data: FixtureMeasurements,
}

#[derive(Deserialize)]
struct FixtureMeasurements {
    timestamp: Vec<f64>,
    co2: Vec<f64>,
    ch4: Vec<f64>,
    h2o: Vec<f64>,
    chamber_t: Vec<f64>,
    chamber_p: Vec<f64>,
}

#[derive(Deserialize)]
struct ExpectedFlux {
    collar: String,
    replicate: String,
    flux_co2_umol_m2_s: f64,
    flux_ch4_nmol_m2_s: f64,
    flux_h2o_umol_m2_s: f64,
    r2_co2: f64,
    r2_ch4: f64,
    r2_h2o: f64,
}

// ---------------------------------------------------------------------------
// Helper: load a fixture file and compute flux
// ---------------------------------------------------------------------------

fn load_fixture_and_compute(fixture_path: &str) -> soil_sensor_toolbox::GasFluxResult {
    let json_str = std::fs::read_to_string(fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read {fixture_path}: {e}"));
    let fixture: FixtureData = serde_json::from_str(&json_str)
        .unwrap_or_else(|e| panic!("Failed to parse {fixture_path}: {e}"));

    // Convert units: ml -> m³, cm² -> m²
    let total_volume_m3 = fixture.total_volume_ml * 1e-6;
    let chamber_area_m2 = fixture.area_cm2 * 1e-4;

    compute_gas_flux(
        &fixture.data.timestamp,
        &fixture.data.co2,
        &fixture.data.ch4,
        &fixture.data.h2o,
        &fixture.data.chamber_t,
        &fixture.data.chamber_p,
        total_volume_m3,
        chamber_area_m2,
    )
}

fn load_expected_fluxes() -> Vec<ExpectedFlux> {
    let json_str = std::fs::read_to_string("tests/fixtures/gas_flux/expected_fluxes.json")
        .expect("Failed to read expected_fluxes.json");
    serde_json::from_str(&json_str).expect("Failed to parse expected_fluxes.json")
}

// ---------------------------------------------------------------------------
// Synthetic tests (moved from former lib.rs gas_flux_tests module)
// ---------------------------------------------------------------------------

#[test]
fn test_flux_known_values() {
    // Synthetic test: constant concentrations -> zero flux
    let n = 100;
    let timestamps: Vec<f64> = (1..=n).map(|i| i as f64).collect();
    let co2: Vec<f64> = vec![400.0; n];
    let ch4: Vec<f64> = vec![2000.0; n];
    let h2o: Vec<f64> = vec![16.0; n];
    let temp: Vec<f64> = vec![25.0; n];
    let pressure: Vec<f64> = vec![91.0; n];
    let volume = 16852.1e-6; // m3
    let area = 318e-4; // m2

    let result = compute_gas_flux(
        &timestamps,
        &co2,
        &ch4,
        &h2o,
        &temp,
        &pressure,
        volume,
        area,
    );

    assert!(
        result.flux_co2_umol_m2_s.abs() < 1e-10,
        "constant CO2 -> zero flux, got {}",
        result.flux_co2_umol_m2_s
    );
    assert!(
        result.flux_ch4_nmol_m2_s.abs() < 1e-10,
        "constant CH4 -> zero flux, got {}",
        result.flux_ch4_nmol_m2_s
    );
}

#[test]
fn test_flux_linear_co2_increase() {
    // CO2 increases linearly: 400 ppm at t=0, rising 0.1 ppm/s
    // slope_co2 in mol/mol/s = 0.1e-6
    // T = 25 deg C = 298.15 K, P = 91 kPa = 91000 Pa
    // flux = slope * (P/(R*T)) * (V/A) * 1e6
    let n = 300;
    let timestamps: Vec<f64> = (1..=n).map(|i| i as f64).collect();
    let co2: Vec<f64> = timestamps.iter().map(|&t| 400.0 + 0.1 * t).collect();
    let ch4: Vec<f64> = vec![2000.0; n];
    let h2o: Vec<f64> = vec![16.0; n];
    let temp: Vec<f64> = vec![25.0; n];
    let pressure: Vec<f64> = vec![91.0; n];
    let volume = 16852.1e-6;
    let area = 318e-4;

    let result = compute_gas_flux(
        &timestamps,
        &co2,
        &ch4,
        &h2o,
        &temp,
        &pressure,
        volume,
        area,
    );

    // Manual calculation:
    let r_gas = 8.314;
    let t_k = 298.15;
    let p_pa = 91000.0;
    let slope_mol = 0.1e-6; // 0.1 ppm/s = 0.1e-6 mol/mol/s
    let expected_co2 = slope_mol * (p_pa / (r_gas * t_k)) * (volume / area) * 1e6;

    assert!(
        (result.flux_co2_umol_m2_s - expected_co2).abs() < 0.01,
        "CO2 flux: expected {expected_co2:.4}, got {:.4}",
        result.flux_co2_umol_m2_s
    );
    assert!(
        result.r2_co2 > 0.999,
        "R2 should be ~1.0 for perfect line, got {}",
        result.r2_co2
    );
    assert!(
        result.flux_ch4_nmol_m2_s.abs() < 1e-6,
        "CH4 should be ~0, got {}",
        result.flux_ch4_nmol_m2_s
    );
}

// ---------------------------------------------------------------------------
// Real Balmoos data tests (col_1 REP_1, col_1 REP_2, col_9 REP_1)
// ---------------------------------------------------------------------------

const FLUX_TOLERANCE: f64 = 0.01;
const R2_TOLERANCE: f64 = 0.001;

/// Compare a computed `GasFluxResult` against expected values with tolerances.
fn assert_flux_close(
    label: &str,
    result: &soil_sensor_toolbox::GasFluxResult,
    expected: &ExpectedFlux,
) {
    assert!(
        (result.flux_co2_umol_m2_s - expected.flux_co2_umol_m2_s).abs() < FLUX_TOLERANCE,
        "{label}: CO2 flux expected {:.6}, got {:.6}, diff {:.6}",
        expected.flux_co2_umol_m2_s,
        result.flux_co2_umol_m2_s,
        (result.flux_co2_umol_m2_s - expected.flux_co2_umol_m2_s).abs()
    );
    assert!(
        (result.flux_ch4_nmol_m2_s - expected.flux_ch4_nmol_m2_s).abs() < FLUX_TOLERANCE,
        "{label}: CH4 flux expected {:.6}, got {:.6}, diff {:.6}",
        expected.flux_ch4_nmol_m2_s,
        result.flux_ch4_nmol_m2_s,
        (result.flux_ch4_nmol_m2_s - expected.flux_ch4_nmol_m2_s).abs()
    );
    assert!(
        (result.flux_h2o_umol_m2_s - expected.flux_h2o_umol_m2_s).abs() < FLUX_TOLERANCE,
        "{label}: H2O flux expected {:.6}, got {:.6}, diff {:.6}",
        expected.flux_h2o_umol_m2_s,
        result.flux_h2o_umol_m2_s,
        (result.flux_h2o_umol_m2_s - expected.flux_h2o_umol_m2_s).abs()
    );
    assert!(
        (result.r2_co2 - expected.r2_co2).abs() < R2_TOLERANCE,
        "{label}: R2 CO2 expected {:.6}, got {:.6}, diff {:.6}",
        expected.r2_co2,
        result.r2_co2,
        (result.r2_co2 - expected.r2_co2).abs()
    );
    assert!(
        (result.r2_ch4 - expected.r2_ch4).abs() < R2_TOLERANCE,
        "{label}: R2 CH4 expected {:.6}, got {:.6}, diff {:.6}",
        expected.r2_ch4,
        result.r2_ch4,
        (result.r2_ch4 - expected.r2_ch4).abs()
    );
    assert!(
        (result.r2_h2o - expected.r2_h2o).abs() < R2_TOLERANCE,
        "{label}: R2 H2O expected {:.6}, got {:.6}, diff {:.6}",
        expected.r2_h2o,
        result.r2_h2o,
        (result.r2_h2o - expected.r2_h2o).abs()
    );
}

#[test]
fn test_balmoos_col_1_rep_1() {
    let result = load_fixture_and_compute("tests/fixtures/gas_flux/col_1_rep_1.json");
    let expected_all = load_expected_fluxes();
    let expected = expected_all
        .iter()
        .find(|e| e.collar == "col_1" && e.replicate == "REP_1")
        .expect("col_1 REP_1 not found in expected fluxes");

    println!(
        "col_1 REP_1 computed: CO2={:.4} CH4={:.4} H2O={:.4}",
        result.flux_co2_umol_m2_s, result.flux_ch4_nmol_m2_s, result.flux_h2o_umol_m2_s
    );
    println!(
        "col_1 REP_1 expected: CO2={:.4} CH4={:.4} H2O={:.4}",
        expected.flux_co2_umol_m2_s, expected.flux_ch4_nmol_m2_s, expected.flux_h2o_umol_m2_s
    );

    assert_flux_close("col_1 REP_1", &result, expected);
}

#[test]
fn test_balmoos_col_1_rep_2() {
    let result = load_fixture_and_compute("tests/fixtures/gas_flux/col_1_rep_2.json");
    let expected_all = load_expected_fluxes();
    let expected = expected_all
        .iter()
        .find(|e| e.collar == "col_1" && e.replicate == "REP_2")
        .expect("col_1 REP_2 not found in expected fluxes");

    println!(
        "col_1 REP_2 computed: CO2={:.4} CH4={:.4} H2O={:.4}",
        result.flux_co2_umol_m2_s, result.flux_ch4_nmol_m2_s, result.flux_h2o_umol_m2_s
    );
    println!(
        "col_1 REP_2 expected: CO2={:.4} CH4={:.4} H2O={:.4}",
        expected.flux_co2_umol_m2_s, expected.flux_ch4_nmol_m2_s, expected.flux_h2o_umol_m2_s
    );

    assert_flux_close("col_1 REP_2", &result, expected);
}

#[test]
fn test_balmoos_col_9_rep_1() {
    let result = load_fixture_and_compute("tests/fixtures/gas_flux/col_9_rep_1.json");
    let expected_all = load_expected_fluxes();
    let expected = expected_all
        .iter()
        .find(|e| e.collar == "col_9" && e.replicate == "REP_1")
        .expect("col_9 REP_1 not found in expected fluxes");

    println!(
        "col_9 REP_1 computed: CO2={:.4} CH4={:.4} H2O={:.4}",
        result.flux_co2_umol_m2_s, result.flux_ch4_nmol_m2_s, result.flux_h2o_umol_m2_s
    );
    println!(
        "col_9 REP_1 expected: CO2={:.4} CH4={:.4} H2O={:.4}",
        expected.flux_co2_umol_m2_s, expected.flux_ch4_nmol_m2_s, expected.flux_h2o_umol_m2_s
    );

    assert_flux_close("col_9 REP_1", &result, expected);
}

/// Comprehensive test that runs all three Balmoos test cases in sequence.
#[test]
fn test_balmoos_all_cases() {
    let expected_all = load_expected_fluxes();
    let fixtures = [
        ("col_1_rep_1", "tests/fixtures/gas_flux/col_1_rep_1.json"),
        ("col_1_rep_2", "tests/fixtures/gas_flux/col_1_rep_2.json"),
        ("col_9_rep_1", "tests/fixtures/gas_flux/col_9_rep_1.json"),
    ];

    let mut passed = 0;

    for (label, fixture_path) in &fixtures {
        let result = load_fixture_and_compute(fixture_path);

        // Find expected entry by matching collar and replicate from the label
        let parts: Vec<&str> = label.split('_').collect();
        let collar = format!("{}_{}", parts[0], parts[1]);
        let replicate = format!("{}_{}", parts[2].to_uppercase(), parts[3]);
        let expected = expected_all
            .iter()
            .find(|e| e.collar == collar && e.replicate == replicate)
            .unwrap_or_else(|| panic!("{label}: not found in expected fluxes"));

        println!(
            "{label}: CO2={:.4} (exp {:.4}), CH4={:.4} (exp {:.4}), H2O={:.4} (exp {:.4})",
            result.flux_co2_umol_m2_s,
            expected.flux_co2_umol_m2_s,
            result.flux_ch4_nmol_m2_s,
            expected.flux_ch4_nmol_m2_s,
            result.flux_h2o_umol_m2_s,
            expected.flux_h2o_umol_m2_s,
        );

        assert_flux_close(label, &result, expected);
        passed += 1;
    }

    println!(
        "\nAll {passed}/{} Balmoos test cases passed.",
        fixtures.len()
    );
}
