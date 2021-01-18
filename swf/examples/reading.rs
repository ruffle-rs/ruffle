use std::fs::File;
use std::io::BufReader;

fn main() {
    let file = File::open("tests/swfs/SimpleRedBackground.swf").unwrap();
    let reader = BufReader::new(file);
    let swf_stream = swf::read_swf_header(reader).unwrap();
    let swf = swf::read_swf(&swf_stream).unwrap();
    println!("The SWF has {} frame(s).", swf.header.num_frames);
    println!("The SWF has {} tag(s).", swf.tags.len());
}
