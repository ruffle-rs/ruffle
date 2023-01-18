# run this file from ruffle root dir (not the docker dir) like 
# rm -rf web/docker/docker_builds/*
# docker build --tag ruffle-web-docker -f web/docker/Dockerfile .
# docker cp $(docker create ruffle-web-docker:latest):/ruffle/web/packages web/docker/docker_builds/packages
FROM ubuntu:22.04
ENV DEBIAN_FRONTEND=noninteractive 
RUN apt-get update -y 
RUN apt-get -y full-upgrade
RUN apt-get install -y wget
RUN wget 'https://deb.nodesource.com/setup_lts.x' --quiet -O- | bash
RUN apt-get update
RUN apt-get install -y \
 git pkg-config openssl libssl-dev gcc gzip tar \
 default-jdk default-jre \
  nodejs
# ... sometimes github.com returns 404 for this one? so.. hacky-try again a few times. ref https://github.com/ruffle-rs/ruffle/pull/9121#issuecomment-1386209670
RUN bash -c ' \
for i in {1..10}; \
do \
wget 'https://github.com/WebAssembly/binaryen/releases/download/version_111/binaryen-version_111-x86_64-linux.tar.gz'; \
if [ $? -eq 0 ]; then break; fi; \
done; \
if [ $? -ne 0 ]; then echo FAILED to download binaryen \$i times 1>&2; exit 1; fi; \
tar xfvz binaryen-version_111-x86_64-linux.tar.gz; \
'
RUN cp -rf binaryen-version_111/* /usr/local/
RUN wget 'https://sh.rustup.rs' --quiet -O- | sh -s -- -y
# RUN source "$HOME/.cargo/env"
# source to modify env doesn't work with docker it seems :( so add cargo to PATH manually:
ENV PATH="/root/.cargo/bin:$PATH"
RUN rustup update
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-bindgen-cli --version 0.2.83
COPY . ruffle
WORKDIR ruffle/web
RUN npm install
RUN npm run build:dual-wasm
