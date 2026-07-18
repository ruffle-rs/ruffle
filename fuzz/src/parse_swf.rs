#[inline(always)]
pub fn parse_swf(data: &[u8]) {
    let Ok(swf_buf) = swf::decompress_swf(data) else {
        return;
    };
    swf::parse_swf(&swf_buf).ok();
}
