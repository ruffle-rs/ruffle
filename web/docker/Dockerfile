# run this file from ruffle root dir (not the docker dir) like 
# rm -rf web/docker/docker_builds/*
# docker build --tag ruffle-web-docker -f web/docker/Dockerfile .
# docker cp $(docker create ruffle-web-docker:latest):/ruffle/web/packages web/docker/docker_builds/packages
FROM ubuntu:22.04
ENV DEBIAN_FRONTEND=noninteractive 
RUN apt-get update -y 
RUN apt-get -y full-upgrade
RUN apt-get install -y apt-utils
RUN apt-get install -y wget git openssl libssl-dev gcc gzip tar default-jre-headless pkg-config
RUN wget 'https://deb.nodesource.com/setup_lts.x' --quiet -O- | bash
RUN apt-get install -y nodejs
RUN wget 'https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh';
RUN bash ./Miniconda3-latest-Linux-x86_64.sh -b -p /miniconda;
ENV PATH="/miniconda/bin:$PATH"
RUN conda install -y -c conda-forge binaryen
RUN wget 'https://sh.rustup.rs' --quiet -O- | sh -s -- -y --profile minimal --target wasm32-unknown-unknown
# RUN source "$HOME/.cargo/env"
# source to modify env doesn't work with docker it seems :( so add cargo to PATH manually:
ENV PATH="/root/.cargo/bin:$PATH"
RUN cargo install wasm-bindgen-cli --version 0.2.87
COPY . ruffle
WORKDIR ruffle/web
RUN npm install --omit=optional
RUN npm run build:repro
