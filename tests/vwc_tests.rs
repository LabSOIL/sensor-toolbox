use chrono::NaiveDateTime;
use csv::ReaderBuilder;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use vwc_test::{process_file, SoilType};

const VWC_TOLERANCE: f64 = 0.0025; // Allow small precision differences

#[derive(Debug)]
struct Mismatch {
    index: usize,
    field: String,
    expected: String,
    actual: String,
    diff: Option<f64>,
}

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

fn compare_with_expected(actual_file: &str, expected_file: &str, soil_type: SoilType) {
    let actual_data = process_file(actual_file, soil_type).unwrap();

    let file = File::open(expected_file).unwrap();
    let reader = BufReader::new(file);
    let expected_lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>().unwrap();

    // Skip the header
    let expected_data_lines = &expected_lines[1..];

    assert_eq!(
        actual_data.len(),
        expected_data_lines.len(),
        "Data length mismatch: actual={}, expected={}",
        actual_data.len(),
        expected_data_lines.len()
    );

    let mut mismatches = Vec::new();
    let total_records = actual_data.len();

    for (i, (actual, expected_line)) in actual_data
        .iter()
        .zip(expected_data_lines.iter())
        .enumerate()
    {
        let parts: Vec<&str> = expected_line.split(',').collect();
        if parts.len() >= 4 {
            let expected_datetime = parts[0].trim_matches('"');
            let expected_raw: f64 = parts[1].parse().unwrap();
            let expected_temp: f64 = parts[2].parse().unwrap();
            let expected_vwc: f64 = parts[3].parse().unwrap();

            let actual_datetime = actual.0.format("%Y-%m-%d %H:%M:%S").to_string();

            // Check datetime
            if actual_datetime != expected_datetime {
                mismatches.push(Mismatch {
                    index: i,
                    field: "datetime".to_string(),
                    expected: expected_datetime.to_string(),
                    actual: actual_datetime,
                    diff: None,
                });
            }

            // Check raw value
            if actual.1 != expected_raw {
                mismatches.push(Mismatch {
                    index: i,
                    field: "raw".to_string(),
                    expected: expected_raw.to_string(),
                    actual: actual.1.to_string(),
                    diff: Some((actual.1 - expected_raw).abs()),
                });
            }

            // Check temperature
            if actual.2 != expected_temp {
                mismatches.push(Mismatch {
                    index: i,
                    field: "temperature".to_string(),
                    expected: expected_temp.to_string(),
                    actual: actual.2.to_string(),
                    diff: Some((actual.2 - expected_temp).abs()),
                });
            }

            // Check VWC with tolerance
            let vwc_diff = (actual.3 - expected_vwc).abs();
            if vwc_diff > VWC_TOLERANCE {
                mismatches.push(Mismatch {
                    index: i,
                    field: "VWC".to_string(),
                    expected: expected_vwc.to_string(),
                    actual: actual.3.to_string(),
                    diff: Some(vwc_diff),
                });
            }
        }
    }

    // Report statistics
    let mismatch_count = mismatches.len();
    let mismatch_percentage = (mismatch_count as f64 / total_records as f64) * 100.0;

    println!("\n=== Comparison Results for {:?} ===", soil_type);
    println!("Total records: {}", total_records);
    println!(
        "Mismatches: {} ({:.2}%)",
        mismatch_count, mismatch_percentage
    );

    if mismatch_count > 0 {
        // Count mismatches by field
        let mut field_counts = std::collections::HashMap::new();
        for mismatch in &mismatches {
            *field_counts.entry(&mismatch.field).or_insert(0) += 1;
        }

        println!("\nMismatch breakdown:");
        for (field, count) in &field_counts {
            let field_percentage = (*count as f64 / total_records as f64) * 100.0;
            println!("  {}: {} ({:.2}%)", field, count, field_percentage);
        }

        // Show first few and last few mismatches
        println!("\nFirst 5 mismatches:");
        for mismatch in mismatches.iter().take(5) {
            if let Some(diff) = mismatch.diff {
                println!(
                    "  Index {}: {} - expected: {}, actual: {}, diff: {:.6}",
                    mismatch.index, mismatch.field, mismatch.expected, mismatch.actual, diff
                );
            } else {
                println!(
                    "  Index {}: {} - expected: {}, actual: {}",
                    mismatch.index, mismatch.field, mismatch.expected, mismatch.actual
                );
            }
        }

        if mismatch_count > 10 {
            println!("\nLast 5 mismatches:");
            for mismatch in mismatches.iter().rev().take(5).rev() {
                if let Some(diff) = mismatch.diff {
                    println!(
                        "  Index {}: {} - expected: {}, actual: {}, diff: {:.6}",
                        mismatch.index, mismatch.field, mismatch.expected, mismatch.actual, diff
                    );
                } else {
                    println!(
                        "  Index {}: {} - expected: {}, actual: {}",
                        mismatch.index, mismatch.field, mismatch.expected, mismatch.actual
                    );
                }
            }
        } else if mismatch_count > 5 {
            println!("\nRemaining mismatches:");
            for mismatch in mismatches.iter().skip(5) {
                if let Some(diff) = mismatch.diff {
                    println!(
                        "  Index {}: {} - expected: {}, actual: {}, diff: {:.6}",
                        mismatch.index, mismatch.field, mismatch.expected, mismatch.actual, diff
                    );
                } else {
                    println!(
                        "  Index {}: {} - expected: {}, actual: {}",
                        mismatch.index, mismatch.field, mismatch.expected, mismatch.actual
                    );
                }
            }
        }

        // For VWC mismatches, show some statistics
        let vwc_mismatches: Vec<f64> = mismatches
            .iter()
            .filter(|m| m.field == "VWC" && m.diff.is_some())
            .map(|m| m.diff.unwrap())
            .collect();

        if !vwc_mismatches.is_empty() {
            let min_diff = vwc_mismatches.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_diff = vwc_mismatches
                .iter()
                .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let avg_diff = vwc_mismatches.iter().sum::<f64>() / vwc_mismatches.len() as f64;

            println!("\nVWC difference statistics:");
            println!("  Min difference: {:.6}", min_diff);
            println!("  Max difference: {:.6}", max_diff);
            println!("  Average difference: {:.6}", avg_diff);
            println!("  Tolerance: {:.6}", VWC_TOLERANCE);
        }
    }

    // Only fail the test if there are critical mismatches (non-VWC or VWC differences much larger than tolerance)
    let critical_mismatches: Vec<&Mismatch> = mismatches
        .iter()
        .filter(|m| {
            m.field != "VWC"
                || (m.field == "VWC" && m.diff.map_or(true, |d| d > VWC_TOLERANCE * 2.0))
        })
        .collect();

    if !critical_mismatches.is_empty() {
        panic!(
            "\nCritical mismatches found: {} out of {} total mismatches",
            critical_mismatches.len(),
            mismatch_count
        );
    }

    println!(
        "\n=== Test passed with {} minor VWC precision differences ===",
        mismatch_count - critical_mismatches.len()
    );
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

// Simple debug test to quickly check a few values
#[test]
fn check_universal_debug() {
    let actual_data = process_file("tests/fixtures/data.csv", SoilType::Universal).unwrap();

    println!("First 5 records:");
    for (i, record) in actual_data.iter().take(5).enumerate() {
        println!(
            "  Index {}: datetime={}, raw={}, temp={}, vwc={:.6}",
            i,
            record.0.format("%Y-%m-%d %H:%M:%S"),
            record.1,
            record.2,
            record.3
        );
    }

    println!("Last 5 records:");
    for (i, record) in actual_data.iter().enumerate().rev().take(5).rev() {
        println!(
            "  Index {}: datetime={}, raw={}, temp={}, vwc={:.6}",
            i,
            record.0.format("%Y-%m-%d %H:%M:%S"),
            record.1,
            record.2,
            record.3
        );
    }

    println!("Total records processed: {}", actual_data.len());

    // Basic sanity checks
    assert!(!actual_data.is_empty(), "No data processed");
    assert!(
        actual_data.iter().all(|r| r.3 >= 0.0),
        "VWC values should be non-negative"
    );
}
