#!/bin/bash
set -euxo pipefail

rm -rf /tmp/ruffle-website-update
mkdir /tmp/ruffle-website-update

cargo run --package stub-report /tmp/ruffle-website-update/implementation.json

cd /tmp/ruffle-website-update/
git clone https://github.com/ruffle-rs/api-report
cd api-report
cargo run -- -s avm2_specification.json -i ../implementation.json -o ../report.json

cd ../

git clone https://github.com/ruffle-rs/ruffle-rs.github.io
cd ruffle-rs.github.io

if cmp -s "../report.json" "src/app/compatibility/avm2/report.json"; then
	echo "Report is unchanged, exiting"
	exit 0
fi

echo "Report is changed - pushing commit"
cp ../report.json src/app/compatibility/avm2/report.json
git add src/app/compatibility/avm2/report.json
git commit -m "Update AVM2 report from https://github.com/ruffle-rs/ruffle/commit/${GITHUB_SHA}"
