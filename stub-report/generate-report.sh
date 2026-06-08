#!/bin/bash

# Generates the AVM2 report and writes it to 'avm2_report.json'

set -euxo pipefail

cargo run --locked --package stub-report -- --avm2-report avm2_report.json
