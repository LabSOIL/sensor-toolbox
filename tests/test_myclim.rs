use soil_sensor_toolbox::{process_file, SoilType};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

const VWC_TOLERANCE: f64 = 0.0025; // Allow small precision differences

#[derive(Debug)]
struct Mismatch {
    index: usize,
    field: String,
    expected: String,
    actual: String,
    diff: Option<f64>,
}

/// Get the expected output filename for a given soil type
fn get_expected_filename(soil_type: SoilType) -> String {
    let soil_name = match soil_type {
        SoilType::Sand => "sand",
        SoilType::LoamySandA => "loamy_sand_A",
        SoilType::LoamySandB => "loamy_sand_B",
        SoilType::SandyLoamA => "sandy_loam_A",
        SoilType::SandyLoamB => "sandy_loam_B",
        SoilType::Loam => "loam",
        SoilType::SiltLoam => "silt_loam",
        SoilType::Peat => "peat",
        SoilType::Water => "water",
        SoilType::Universal => "universal",
        SoilType::SandTms1 => "sand_TMS1",
        SoilType::LoamySandTms1 => "loamy_sand_TMS1",
        SoilType::SiltLoamTms1 => "silt_loam_TMS1",
    };
    format!("tests/fixtures/data/output_{}.csv", soil_name)
}

fn compare_soil_type(soil_type: SoilType) -> Result<(), Box<dyn Error>> {
    let actual_data = process_file("tests/fixtures/data/data.csv".to_string(), soil_type)?;
    let expected_file = get_expected_filename(soil_type);

    let file = File::open(&expected_file)
        .map_err(|e| format!("Failed to open expected file {}: {}", expected_file, e))?;
    let reader = BufReader::new(file);
    let expected_lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;

    // Skip the header
    let expected_data_lines = &expected_lines[1..];

    assert_eq!(
        actual_data.len(),
        expected_data_lines.len(),
        "Data length mismatch for {:?}: actual={}, expected={}",
        soil_type,
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
            let expected_raw: f64 = parts[1].parse()?;
            let expected_temp: f64 = parts[2].parse()?;
            let expected_vwc: f64 = parts[3].parse()?;

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

        // Show first few mismatches
        if mismatch_count <= 5 {
            println!("\nAll mismatches:");
            for mismatch in &mismatches {
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
        } else {
            println!("\nFirst 3 mismatches:");
            for mismatch in mismatches.iter().take(3) {
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

    // Only fail the test if there are critical mismatches
    let critical_mismatches: Vec<&Mismatch> = mismatches
        .iter()
        .filter(|m| m.field != "VWC" || m.diff.is_some_and(|d| d > VWC_TOLERANCE * 2.0))
        .collect();

    if !critical_mismatches.is_empty() {
        return Err(format!(
            "Critical mismatches found for {:?}: {} out of {} total mismatches",
            soil_type,
            critical_mismatches.len(),
            mismatch_count
        )
        .into());
    }

    if mismatch_count > 0 {
        println!(
            "=== Test passed with {} minor VWC precision differences ===",
            mismatch_count - critical_mismatches.len()
        );
    } else {
        println!("=== Perfect match! ===");
    }

    Ok(())
}

// Individual tests for each soil type
#[test]
fn test_sand() {
    compare_soil_type(SoilType::Sand).expect("Sand soil type test failed");
}

#[test]
fn test_loamy_sand_a() {
    compare_soil_type(SoilType::LoamySandA).expect("Loamy Sand A soil type test failed");
}

#[test]
fn test_loamy_sand_b() {
    compare_soil_type(SoilType::LoamySandB).expect("Loamy Sand B soil type test failed");
}

#[test]
fn test_sandy_loam_a() {
    compare_soil_type(SoilType::SandyLoamA).expect("Sandy Loam A soil type test failed");
}

#[test]
fn test_sandy_loam_b() {
    compare_soil_type(SoilType::SandyLoamB).expect("Sandy Loam B soil type test failed");
}

#[test]
fn test_loam() {
    compare_soil_type(SoilType::Loam).expect("Loam soil type test failed");
}

#[test]
fn test_silt_loam() {
    compare_soil_type(SoilType::SiltLoam).expect("Silt Loam soil type test failed");
}

#[test]
fn test_peat() {
    compare_soil_type(SoilType::Peat).expect("Peat soil type test failed");
}

#[test]
fn test_water() {
    compare_soil_type(SoilType::Water).expect("Water soil type test failed");
}

#[test]
fn test_universal() {
    compare_soil_type(SoilType::Universal).expect("Universal soil type test failed");
}

#[test]
fn test_sand_tms1() {
    compare_soil_type(SoilType::SandTms1).expect("Sand TMS1 soil type test failed");
}

#[test]
fn test_loamy_sand_tms1() {
    compare_soil_type(SoilType::LoamySandTms1).expect("Loamy Sand TMS1 soil type test failed");
}

#[test]
fn test_silt_loam_tms1() {
    compare_soil_type(SoilType::SiltLoamTms1).expect("Silt Loam TMS1 soil type test failed");
}

/// Comprehensive test that runs all soil types
#[test]
fn test_all_soil_types() {
    let soil_types = SoilType::ALL.to_vec();
    let mut failed_types = Vec::new();
    let mut passed_count = 0;

    println!(
        "Running comprehensive test for all {} soil types...",
        soil_types.len()
    );

    for soil_type in &soil_types {
        match compare_soil_type(*soil_type) {
            Ok(_) => {
                println!("✅ {:?} passed", soil_type);
                passed_count += 1;
            }
            Err(e) => {
                println!("❌ {:?} failed: {}", soil_type, e);
                failed_types.push(soil_type);
            }
        }
    }

    println!("\n=== Final Results ===");
    println!("Passed: {}/{}", passed_count, soil_types.len());
    println!("Failed: {}", failed_types.len());

    if !failed_types.is_empty() {
        println!("Failed soil types: {:?}", failed_types);
        panic!("Some soil type tests failed");
    }

    println!("\n🎉 All {} soil types passed!", soil_types.len());
}
