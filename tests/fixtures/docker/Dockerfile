# 1. Base image with tidyverse (dplyr, purrr, etc.)
FROM rocker/tidyverse:4.2.2

# 2. System libs for installing from GitHub
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      libcurl4-openssl-dev \
      libssl-dev \
      libxml2-dev \
 && rm -rf /var/lib/apt/lists/*

# 3. Install remotes and then myClim (with its deps)
RUN Rscript -e "install.packages('remotes', repos='https://cloud.r-project.org')" \
 && Rscript -e "remotes::install_github('ibot-geoecology/myClim', dependencies = TRUE)"

# 4. Copy our R script in
COPY script_all_soils.R /usr/local/bin/script_all_soils.R
RUN chmod +x /usr/local/bin/script_all_soils.R

# 5. Use /data as workdir, so you can mount your .csv there
WORKDIR /data

# 6. When container runs, execute script_all_soils.R
ENTRYPOINT ["Rscript", "/usr/local/bin/script_all_soils.R"]
