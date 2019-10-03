use swf::*;

fn main() {
    let swf = Swf {
        header: Header {
            version: 6,
            compression: Compression::Zlib,
            stage_size: Rectangle {
                x_min: Twips::from_pixels(0.0),
                x_max: Twips::from_pixels(400.0),
                y_min: Twips::from_pixels(0.0),
                y_max: Twips::from_pixels(400.0),
            },
            frame_rate: 60.0,
            num_frames: 1,
        },
        tags: vec![
            Tag::SetBackgroundColor(Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            }),
            Tag::ShowFrame,
        ],
    };
    let file = std::fs::File::create("tests/swfs/SimpleRedBackground.swf").unwrap();
    let writer = std::io::BufWriter::new(file);
    swf::write_swf(&swf, writer).unwrap();
}
