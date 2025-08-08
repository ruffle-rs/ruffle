# pbasm Integration Tests

This directory contains integration tests for `pbasm`.
Each directory with `test.toml` contains one integration test.

## `test.toml`

```toml
# Type of the test, either 'assembly', 'disassembly', or 'roundtrip'.
type = "roundtrip"
# If set to true, the test will be ignored.
ignore = false
```
