/*
 * VWC (Volumetric Water Content) Calculation Tool
 *
 * This implementation is based on the myClim R package algorithms and coefficients.
 * See lib.rs for full license attribution.
 */

use csv::WriterBuilder;
use soil_sensor_toolbox::{process_file, SoilType};
use std::env;
use std::process;

fn print_usage() {
    println!("Usage: soil-sensor-toolbox <input_file> <soil_type>");
    println!("\nAvailable soil types:");
    for soil in &SoilType::ALL {
        println!("  {}", soil.as_str());
    }
    println!("\nExample:");
    println!("  soil-sensor-toolbox data.csv universal");
    println!("  soil-sensor-toolbox data.csv peat");
}

fn process_args(args: &[String]) -> Result<(String, SoilType), String> {
    if args.len() != 3 {
        return Err("Invalid number of arguments".to_string());
    }

    let input_file = args[1].clone();
    let soil_type = match args[2].as_str().try_into() {
        Ok(soil) => soil,
        Err(e) => {
            eprintln!("Error: {e}");
            println!();
            print_usage();

            process::exit(1);
        }
    };

    Ok((input_file, soil_type))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        print_usage();
        process::exit(1);
    }

    let (input_file, soil_type) = process_args(&args)?;
    let records = process_file(input_file, soil_type)?;
    let mut wtr = WriterBuilder::new()
        .delimiter(b';')
        .from_path("output.csv")?;
    wtr.write_record(["datetime", "raw", "temp", "VWC_moisture"])?;
    for (dt, raw, temp, vwc) in records {
        wtr.write_record(&[
            dt.format("%Y.%m.%d %H:%M").to_string(),
            raw.to_string(),
            temp.to_string(),
            format!("{vwc:.6}"),
        ])?;
    }
    wtr.flush()?;
    println!("wrote output.csv");

    Ok(())
}
