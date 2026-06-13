<div align="center">

# is_wine

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/dSyncro/is_wine/blob/main/README.md)
![Version](https://img.shields.io/badge/version-0.1.2-green)

A library to easily check if the current app is running under wine.

</div>

## 📖 Table of Contents

- [Dependencies and requirements](#dependencies-and-requirements)
- [Getting started](#getting-started)
- [Usage](#usage)
- [Acknowledgement](#acknowledgement)
- [Side notes](#side-notes)

## Dependencies and requirements

This library has no external nor prior dependency and can run under `no_std` environments.

## Getting Started

Just add the library to your project

```bash
cargo add is_wine
```

Import it and that's it!

## Usage

Here is an example usage of `is_wine`

```rust
let is_wine = is_wine(); // Returns true under wine, false elsewise, panics on failure.

let is_wine = is_wine_lax(); // Returns true under wine, false elsewise or on failure.

let is_wine = try_is_wine(); // Returns a type-safe result.
```

## Acknowledgement

After a rapid check on [crates.io](https://crates.io) I noticed there is a similar crate performing the same task:

- [winecheck](https://crates.io/crates/winecheck) (Rust)

If you find it useful please support it too!

## Side notes

Please keep in mind that at the moment this is a side project developed with no planned continuity nor schedule. Therefore _support, fixes and new features can not be guaranteed_.

As stated in the [LICENSE](https://github.com/dSyncro/is_wine/blob/master/LICENSE), _no contributor must be considered liable for the use of this project_.
