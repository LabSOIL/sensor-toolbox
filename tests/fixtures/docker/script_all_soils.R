#!/usr/bin/env Rscript
#
# VWC (Volumetric Water Content) Calculation Script
# 
# This script uses the myClim R package to calculate VWC for all supported soil types.
# 
# myClim Package Attribution:
# Copyright (C) Institute of Botany of the Czech Academy of Sciences
# Licensed under GPL v2: https://github.com/ibot-geoecology/myClim/blob/main/LICENSE.md
# 
# References:
# - Wild et al. (2019), 10.1016/j.agrformet.2018.12.018
# - Kopecký et al. (2021), 10.1016/j.scitotenv.2020.143785
# - Vlček (2010) Kalibrace vlhkostního čidla TST1 pro minerální a organické půdy
#

# 1. load libraries
library(myClim)
library(dplyr)
library(purrr)

# 2. read all TOMST-format files
tms <- mc_read_files("data.csv", dataformat_name = "TOMST", silent = TRUE)

# 3. clean timestamps etc.
cat("Cleaning data...\n")
tms_clean <- mc_prep_clean(tms, silent = TRUE)

# 4. Get all available soil types from myClim
available_soils <- myClim::mc_data_vwc_parameters$soiltype
cat("Available soil types in myClim:\n")
print(available_soils)
cat("\n")

# 5. for each soiltype, calc VWC and write out
for (soil in available_soils) {
  cat("Processing soil type:", soil, "\n")
  
  # Calculate VWC with frozen2NA=FALSE to match our implementation
  tms_vwc <- mc_calc_vwc(tms_clean, soiltype = soil, frozen2NA = FALSE)

  # flatten the myClim object into a data.frame
  df <- tms_vwc$localities %>%
    map_df(function(loc) {
      loc$loggers %>% map_df(function(logger) {
        tibble(
          datetime       = logger$datetime,
          raw            = logger$sensors$TMS_moist$values,
          temp           = logger$sensors$TMS_T1$values,
          VWC_moisture   = logger$sensors$VWC_moisture$values
        )
      })
    })

  # Clean up soil type name for filename (replace spaces and special chars)
  safe_soil_name <- gsub("[^A-Za-z0-9]", "_", soil)
  output_file <- paste0("output_", safe_soil_name, ".csv")
  
  # write it back to host with proper formatting
  write.csv(df,
            file = output_file,
            row.names = FALSE)
  
  cat("Wrote", nrow(df), "records to", output_file, "\n")
}

# 6. Print coefficient table for reference
cat("\n=== Soil Type Coefficients (from myClim) ===\n")
print(myClim::mc_data_vwc_parameters[c("soiltype", "a", "b", "c")])

# 7. Print temperature correction constants
cat("\n=== Temperature Correction Constants ===\n")
cat("ref_t  =", myClim::mc_const_CALIB_MOIST_REF_T, "\n")
cat("acor_t =", myClim::mc_const_CALIB_MOIST_ACOR_T, "\n") 
cat("wcor_t =", myClim::mc_const_CALIB_MOIST_WCOR_T, "\n")

cat("\nProcessing complete! Generated", length(available_soils), "soil type outputs.\n")