# Run this file from ruffle root dir (not the docker dir) like this:
# rm -rf web/docker/docker_builds/packages/*
# docker build --tag ruffle-web-docker -f web/docker/Dockerfile .
# docker cp $(docker create ruffle-web-docker:latest):/ruffle/web/packages web/docker/docker_builds/packages
FROM node:20

# Getting Miniconda from their website:
RUN wget 'https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh'
RUN bash ./Miniconda3-latest-Linux-x86_64.sh -b -p /miniconda
ENV PATH="/miniconda/bin:$PATH"
RUN conda install -y -c conda-forge binaryen

# Installing Rust using rustup:
RUN wget 'https://sh.rustup.rs' --quiet -O- | sh -s -- -y --profile minimal --target wasm32-unknown-unknown
ENV PATH="/root/.cargo/bin:$PATH"
RUN cargo install wasm-bindgen-cli --version 0.2.92

# Building Ruffle:
COPY . ruffle
WORKDIR ruffle/web
ENV CARGO_FEATURES=jpegxr
ENV WASM_SOURCE=existing
RUN npm ci
RUN npm run build:repro
