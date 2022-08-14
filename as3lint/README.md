# About
This crate provides linting for playerglobals located in `core/src/avm2/globals`.

# Usage
To use the linter, run the command `cargo run -p as3lint`. All warnings will be printed to `stderr`. If there are no warnings, nothing will be printed.

# Customizing
All rules that this linter uses are located in `flexpmd/ruffle-ruleset.xml`. There may be rules that make implementing certain classes in playerglobals 
impossible, so feel free to remove any bad rules.
