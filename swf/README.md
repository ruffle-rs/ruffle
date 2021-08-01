# swf
[![crates.io](https://img.shields.io/crates/v/swf.svg)](https://crates.io/crates/swf)
[![docs.rs](https://docs.rs/swf/badge.svg)](https://docs.rs/swf)

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
let swf_buf = swf::decompress_swf(reader).unwrap();
let swf = swf::parse_swf(&swf_buf).unwrap();
println!("The SWF has {} frame(s).", swf.header.num_frames());
println!("The SWF has {} tag(s).", swf.tags.len());
```

Try `cargo run --example reading` in this repository to run this example.

## Writing

```rust,no_run
use swf::*;
let header = Header {
    compression: Compression::Zlib,
    version: 6,
    stage_size: Rectangle {
        x_min: Twips::from_pixels(0.0), x_max: Twips::from_pixels(400.0),
        y_min: Twips::from_pixels(0.0), y_max: Twips::from_pixels(400.0)
    },
    frame_rate: Fixed8::from_f32(60.0),
    num_frames: 1,
};
let tags = [
    Tag::SetBackgroundColor(Color { r: 255, g: 0, b: 0, a: 255 }),
    Tag::ShowFrame
];
let file = std::fs::File::create("file.swf").unwrap();
let writer = std::io::BufWriter::new(file);
swf::write_swf(&header, &tags, writer).unwrap();
```

Try `cargo run --example writing` in this repository to run this example.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
