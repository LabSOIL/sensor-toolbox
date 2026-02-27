/*
 * Soil Sensor Toolbox
 *
 * A Rust library for processing soil sensor data:
 * - TMS4 moisture/VWC calculations (myClim algorithm)
 * - LI-7810 gas flux calculations (MATLAB reader_gas_Flux_EPFL_may.m algorithm)
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

pub mod gas_flux;
pub mod vwc;

pub use gas_flux::*;
pub use vwc::*;
