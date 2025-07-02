# Soil Sensor Toolbox

VWC (Volumetric Water Content) calculation tool based on myClim R package algorithms.

## Installation

### Install from source
```bash
cargo install --path .
```

### Or install from crates.io (if published)
```bash
cargo install soil-sensor-toolbox
```

## Quick Start

### Build (for development)
```bash
cargo build --release
```

### Run (development)
```bash
cargo run -- <input_file> <soil_type>
```

### Run (after installation)
```bash
soil-sensor-toolbox <input_file> <soil_type>
```

**Example:**
```bash
soil-sensor-toolbox data.csv universal
soil-sensor-toolbox data.csv peat
```

**Available soil types:**
- `sand`
- `loamy_sand_A`
- `loamy_sand_B`
- `sandy_loam_A`
- `sandy_loam_B`
- `loam`
- `silt_loam`
- `peat`
- `water`
- `universal`
- `sand_TMS1`
- `loamy_sand_TMS1`
- `silt_loam_TMS1`

### Input Format
Direct from the TMS4 sensor:
```
0;2023.05.30 06:45;4;22.25;22.25;22.5;354;202;0;
1;2023.05.30 07:00;4;21.75;22;22.125;353;202;0;
2;2023.05.30 07:15;4;21.125;21.5;21.375;351;202;0;
...
```

### Output
Creates `output.csv` with VWC calculations.

## Tests

First you will need to generate the test data:
```bash
cd ./tests/fixtures
./generate_r_data.sh
```

This will generate the output files with VWC for each soil type from the R script `myClim`.

Then run the tests:
```bash
cargo test
```
