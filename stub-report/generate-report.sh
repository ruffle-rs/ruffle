#!/bin/bash

# Generates the AVM2 report and writes it to 'avm2_report.json'

set -euxo pipefail

rm -rf /tmp/ruffle-website-update
mkdir /tmp/ruffle-website-update

cargo run --locked --package stub-report /tmp/ruffle-website-update/implementation.json

git clone https://github.com/ruffle-rs/api-report /tmp/ruffle-website-update/api-report
cargo run --manifest-path /tmp/ruffle-website-update/api-report/Cargo.toml  -- -s /tmp/ruffle-website-update/api-report/avm2_specification.json -i /tmp/ruffle-website-update/implementation.json -o avm2_report.json