use csv::WriterBuilder;
use std::env;
use vwc_test::{process_file, SoilType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = env::args().nth(1).expect("usage: vwc_test <raw.csv>");
    for &(soil, name) in &[
        (SoilType::Peat, "output_peat.csv"),
        (SoilType::Universal, "output_universal.csv"),
    ] {
        let records = process_file(&input, soil)?;
        let mut wtr = WriterBuilder::new().delimiter(b';').from_path(name)?;
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
        println!("wrote {}", name);
    }
    Ok(())
}
