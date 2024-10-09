# Run this file from ruffle root dir (not the docker dir) like this:
# rm -rf web/docker/docker_builds/packages/*
# docker build --tag ruffle-web-docker -f web/docker/Dockerfile .
# docker cp $(docker create ruffle-web-docker:latest):/ruffle/web/packages web/docker/docker_builds/packages
FROM node:20

# Installing wasm-opt from GitHub:
# Keep the version number in sync with the ones in the Actions workflows!
RUN wget --progress=:giga https://github.com/WebAssembly/binaryen/releases/download/version_119/binaryen-version_119-x86_64-linux.tar.gz
RUN tar xzf binaryen-version_119-x86_64-linux.tar.gz binaryen-version_119/bin/wasm-opt
RUN mv binaryen-version_119/bin/wasm-opt /usr/local/bin

# Installing Rust using rustup:
RUN wget 'https://sh.rustup.rs' --quiet -O- | sh -s -- -y --profile minimal --target wasm32-unknown-unknown
ENV PATH="/root/.cargo/bin:$PATH"
# wasm-bindgen-cli version must match wasm-bindgen crate version.
# Be sure to update in test_web.yml, release_nightly.yml, Cargo.toml, and web/README.md as well.
RUN cargo install wasm-bindgen-cli --version 0.2.95

# Building Ruffle:
COPY . ruffle
WORKDIR ruffle/web
ENV CARGO_FEATURES=jpegxr
ENV WASM_SOURCE=existing
RUN npm ci
RUN npm run build:repro
