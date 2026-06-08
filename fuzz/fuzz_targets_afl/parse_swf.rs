fn main() {
    afl::fuzz!(|data: &[u8]| {
        ruffle_fuzz::parse_swf::parse_swf(data);
    });
}
