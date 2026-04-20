# Exporter Integration Tests

This directory contains integration tests for `exporter`.
Each directory with `test.toml` contains one integration test.

## `test.toml`

```toml
# Arguments passed to the exporter.
# Two first arguments (program, SWF) are passed automatically.
args = []
# Path to the SWF file to pass as the first argument, relative to the test directory.
swf = "test.swf"
# If set to true, the test will be ignored.
ignore = false
# If set, the test will expect exporter fails with the following error.
expect_error = "..."
# The directory with input files, i.e. files already residing
# in the current working directory of exporter.
input_dir = "input"
# The directory with output files, i.e. files that we expect
# exporter will produce in the current working directory.
output_dir = "output"
```
