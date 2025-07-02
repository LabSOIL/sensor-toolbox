use chrono::NaiveDateTime;
use csv::ReaderBuilder;
use std::error::Error;
use vwc_test::{process_file, SoilType};

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

#[test]
fn check_universal() -> Result<(), Box<dyn Error>> {
    let expected = load_expected("tests/fixtures/output_universal.csv")?;
    let actual = process_file("tests/fixtures/data.csv", SoilType::Universal)?;
    assert_eq!(expected.len(), actual.len());
    for (i, (ed, er, et, ev)) in expected.iter().enumerate() {
        let (ad, ar, at, av) = actual[i];
        assert_eq!(ed, &ad);
        assert!((er - ar).abs() < 1e-6);
        assert!((et - at).abs() < 1e-6);
        assert!((ev - av).abs() < 1e-6);
    }
    Ok(())
}

#[test]
fn check_peat() -> Result<(), Box<dyn Error>> {
    let expected = load_expected("tests/fixtures/output_peat.csv")?;
    let actual = process_file("tests/fixtures/data.csv", SoilType::Peat)?;
    assert_eq!(expected.len(), actual.len());
    for (i, (ed, er, et, ev)) in expected.iter().enumerate() {
        let (ad, ar, at, av) = actual[i];
        assert_eq!(ed, &ad);
        assert!((er - ar).abs() < 1e-6);
        assert!((et - at).abs() < 1e-6);
        assert!((ev - av).abs() < 1e-6);
    }
    Ok(())
}
