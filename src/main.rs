/*
 * VWC (Volumetric Water Content) Calculation Tool
 *
 * This implementation is based on the myClim R package algorithms and coefficients.
 * See lib.rs for full license attribution.
 */

use csv::WriterBuilder;
use std::env;
use std::process;
use vwc_test::{process_file, SoilType};

fn print_usage() {
    println!("Usage: vwc_test <input_file> <soil_type>");
    println!("\nAvailable soil types:");
    for soil in SoilType::all_types() {
        println!("  {}", soil.to_str());
    }
    println!("\nExample:");
    println!("  vwc_test data.csv universal");
    println!("  vwc_test data.csv peat");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        print_usage();
        process::exit(1);
    }

    let input_file = &args[1];
    let soil_type_str = &args[2];

    let soil_type = match SoilType::from_str(soil_type_str) {
        Ok(soil) => soil,
        Err(e) => {
            eprintln!("Error: {}", e);
            println!();
            print_usage();
            process::exit(1);
        }
    };

    let records = process_file(input_file, soil_type)?;
    let mut wtr = WriterBuilder::new()
        .delimiter(b';')
        .from_path("output.csv")?;
    wtr.write_record(&["datetime", "raw", "temp", "VWC_moisture"])?;
    for (dt, raw, temp, vwc) in records {
        wtr.write_record(&[
            dt.format("%Y.%m.%d %H:%M").to_string(),
            raw.to_string(),
            temp.to_string(),
            format!("{:.6}", vwc),
        ])?;
    }
    wtr.flush()?;
    println!("wrote output.csv");

    Ok(())
}
