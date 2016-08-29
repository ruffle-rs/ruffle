# swf

A Rust library for reading and writing the Adobe Flash SWF file format.

```toml
# Cargo.toml
[dependencies]
swf = "0.1"
```

## Reading

```rust
extern crate swf;

use std::io::BufReader;
use std::fs::File;

fn main() {
    let f = File::open("file.swf").unwrap();
    let reader = BufReader::new(f);
    let swf = swf::read_swf(reader).unwrap();
    println!("The SWF has {} frames", swf.num_frames);
}
```

## Writing

```rust,no_run
extern crate swf;

use std::io::BufWriter;
use std::fs::File;
use swf::*;

fn main() {
    let f = File::create("file.swf").unwrap();
    let writer = BufWriter::new(f);
    let swf = Swf {
        version: 6,
        compression: Compression::Zlib,
        stage_size: Rectangle { x_min: 0f32, x_max: 400f32, y_min: 0f32, y_max: 400f32 },
        frame_rate: 60f32,
        num_frames: 1,
        tags: vec![
            Tag::SetBackgroundColor(Color { r: 255, g: 0, b: 0, a: 255 }),
            Tag::ShowFrame
        ]
    };
    swf::write_swf(&swf, writer).unwrap();
}
```

# License

`swf-rs` is distributed under the terms of the GPLv3 license.

See LICENSE.md for details.