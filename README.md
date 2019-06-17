# swf
[![crates.io](https://img.shields.io/crates/v/swf.svg)](https://crates.io/crates/swf)
[![docs.rs](https://docs.rs/swf/badge.svg)](https://docs.rs/swf)
[![TravisCI](https://travis-ci.org/Herschel/swf-rs.svg?branch=master)](https://travis-ci.org/Herschel/swf-rs)

A Rust library for reading and writing the Adobe Flash SWF file format.

```toml
# Cargo.toml
[dependencies]
swf = "0.1"
```

## Reading

```rust
use std::io::BufReader;
use std::fs::File;

let file = File::open("file.swf").unwrap();
let reader = BufReader::new(file);
let swf = swf::read_swf(reader).unwrap();
println!("The SWF has {} frames", swf.num_frames);
```

## Writing

```rust,no_run
use swf::*;
let swf = Swf {
    header: Header {
        version: 6,
        compression: Compression::Zlib,
        stage_size: Rectangle { 
            x_min: Twips::from_pixels(0.0), x_max: Twips::from_pixels(400.0),
            y_min: Twips::from_pixels(0.0), y_max: Twips::from_pixels(400.0)
        },
        frame_rate: 60.0,
        num_frames: 1,
    },
    tags: vec![
        Tag::SetBackgroundColor(Color { r: 255, g: 0, b: 0, a: 255 }),
        Tag::ShowFrame
    ]
};
let file = std::fs::File::create("file.swf").unwrap();
let writer = std::io::BufWriter::new(file);
swf::write_swf(&swf, writer).unwrap();
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.