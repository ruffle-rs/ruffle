# Run this file from ruffle root dir (not the docker dir) like this:
# rm -rf web/docker/docker_builds/*
# docker build --tag ruffle-web-docker -f web/docker/Dockerfile .
# docker cp $(docker create ruffle-web-docker:latest):/ruffle/web/packages web/docker/docker_builds/packages
FROM ubuntu:22.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update -y
RUN apt-get -y full-upgrade
# Installing dependencies:
RUN apt-get install -y apt-utils
RUN apt-get install -y wget git openssl libssl-dev gcc clang gzip tar default-jre-headless pkg-config
# Installing Node.js from the nodesource repo according to their instructions:
RUN apt-get install -y ca-certificates curl gnupg
RUN mkdir -p /etc/apt/keyrings
RUN curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg
ENV NODE_MAJOR=20
RUN echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_$NODE_MAJOR.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list
RUN apt-get update -y
RUN apt-get install -y nodejs
# Getting Miniconda from their website:
RUN wget 'https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh'
RUN bash ./Miniconda3-latest-Linux-x86_64.sh -b -p /miniconda
ENV PATH="/miniconda/bin:$PATH"
RUN conda install -y -c conda-forge binaryen
# Installing Rust using rustup:
RUN wget 'https://sh.rustup.rs' --quiet -O- | sh -s -- -y --profile minimal --target wasm32-unknown-unknown
ENV PATH="/root/.cargo/bin:$PATH"
RUN cargo install wasm-bindgen-cli --version 0.2.89
# Building Ruffle:
COPY . ruffle
WORKDIR ruffle/web
ENV CARGO_FEATURES=jpegxr
RUN npm install
RUN npm run build:repro
