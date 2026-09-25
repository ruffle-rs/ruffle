#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    ruffle_fuzz::parse_swf::parse_swf(data);
});
