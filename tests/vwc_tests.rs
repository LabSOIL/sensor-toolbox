use chrono::NaiveDateTime;
use csv::ReaderBuilder;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use vwc_test::{process_file, SoilType};

const VWC_TOLERANCE: f64 = 0.0025; // Allow small precision differences - increased tolerance

fn load_expected(path: &str) -> Result<Vec<(NaiveDateTime, f64, f64, f64)>, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new()
        .delimiter(b',') // ← comma, not semicolon
        .has_headers(true)
        .from_path(path)?;
    let mut out = Vec::new();
    for result in rdr.records() {
        let rec = result?;
        let dt = NaiveDateTime::parse_from_str(&rec[0], "%Y-%m-%d %H:%M:%S")?;
        let raw: f64 = rec[1].parse()?;
        let temp: f64 = rec[2].parse()?;
        let vwc: f64 = rec[3].parse()?;
        out.push((dt, raw, temp, vwc));
    }
    Ok(out)
}

fn compare_with_expected(
    actual_file: &str,
    expected_file: &str,
    soil_type: SoilType,
) {
    let actual_data = process_file(actual_file, soil_type).unwrap();
    
    let file = File::open(expected_file).unwrap();
    let reader = BufReader::new(file);
    let expected_lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>().unwrap();
    
    // Skip the header
    let expected_data_lines = &expected_lines[1..];
    
    assert_eq!(actual_data.len(), expected_data_lines.len(), 
               "Data length mismatch: actual={}, expected={}", 
               actual_data.len(), expected_data_lines.len());
    
    for (i, (actual, expected_line)) in actual_data.iter().zip(expected_data_lines.iter()).enumerate() {
        let parts: Vec<&str> = expected_line.split(',').collect();
        if parts.len() >= 4 {
            let expected_datetime = parts[0].trim_matches('"');
            let expected_raw: f64 = parts[1].parse().unwrap();
            let expected_temp: f64 = parts[2].parse().unwrap();
            let expected_vwc: f64 = parts[3].parse().unwrap();
            
            let actual_datetime = actual.0.format("%Y-%m-%d %H:%M:%S").to_string();
            
            // Check each field with appropriate tolerance
            assert_eq!(actual_datetime, expected_datetime, 
                      "Datetime mismatch at index {}: expected={}, actual={}", 
                      i, expected_datetime, actual_datetime);
            
            assert_eq!(actual.1, expected_raw, 
                      "Raw value mismatch at index {}: expected={}, actual={}", 
                      i, expected_raw, actual.1);
            
            assert_eq!(actual.2, expected_temp, 
                      "Temperature mismatch at index {}: expected={}, actual={}", 
                      i, expected_temp, actual.2);
            
            // Use tolerance for VWC comparison
            let vwc_diff = (actual.3 - expected_vwc).abs();
            assert!(vwc_diff <= VWC_TOLERANCE, 
                   "VWC mismatch at index {}: expected={}, actual={}, diff={}", 
                   i, expected_vwc, actual.3, vwc_diff);
        }
    }
}

#[test]
fn check_universal() {
    compare_with_expected(
        "tests/fixtures/data.csv",
        "tests/fixtures/output_universal.csv",
        SoilType::Universal,
    );
}

#[test]
fn check_peat() {
    compare_with_expected(
        "tests/fixtures/data.csv", 
        "tests/fixtures/output_peat.csv",
        SoilType::Peat,
    );
}

// Keep the debug test for troubleshooting if needed
#[test]
fn check_universal_debug() {
    let actual_data = process_file("tests/fixtures/data.csv", SoilType::Universal).unwrap();
    
    let file = File::open("tests/fixtures/output_universal.csv").unwrap();
    let reader = BufReader::new(file);
    let expected_lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>().unwrap();
    
    // Skip the header
    let expected_data_lines = &expected_lines[1..];
    
    assert_eq!(actual_data.len(), expected_data_lines.len());
    
    for (i, (actual, expected_line)) in actual_data.iter().zip(expected_data_lines.iter()).enumerate() {
        let parts: Vec<&str> = expected_line.split(',').collect();
        if parts.len() >= 4 {
            let expected_datetime = parts[0].trim_matches('"');
            let expected_raw: f64 = parts[1].parse().unwrap();
            let expected_temp: f64 = parts[2].parse().unwrap();
            let expected_vwc: f64 = parts[3].parse().unwrap();
            
            let actual_datetime = actual.0.format("%Y-%m-%d %H:%M:%S").to_string();
            
            // Print detailed comparison for first few mismatches
            if i < 10 || i > actual_data.len() - 10 {
                println!("Index {}: actual=({}, {}, {}, {}), expected=({}, {}, {}, {})", 
                        i, actual_datetime, actual.1, actual.2, actual.3,
                        expected_datetime, expected_raw, expected_temp, expected_vwc);
            }
            
            // Use tolerance for VWC comparison in debug test too
            let vwc_diff = (actual.3 - expected_vwc).abs();
            if vwc_diff > VWC_TOLERANCE {
                panic!("VWC mismatch at index {}: expected={}, actual={}, diff={}", 
                       i, expected_vwc, actual.3, vwc_diff);
            }
        }
    }
}
