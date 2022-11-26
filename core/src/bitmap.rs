pub mod bitmap_data;
pub mod turbulence;

/// Determine if a particular bitmap data size is valid.
///
/// This enforces limits on BitmapData as specified in the Flash documentation.
/// Specifically, from <https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/display/BitmapData.html>:
///
/// "In AIR 1.5 and Flash Player 10, the maximum size for a BitmapData object
/// is 8,191 pixels in width or height, and the total number of pixels cannot
/// exceed 16,777,215 pixels. (So, if a BitmapData object is 8,191 pixels wide,
/// it can only be 2,048 pixels high.) In Flash Player 9 and earlier and
/// AIR 1.1 and earlier, the limitation is 2,880 pixels in height and 2,880 in
/// width. Starting with AIR 3 and Flash player 11, the size limits for a
/// BitmapData object have been removed. The maximum size of a bitmap is now
/// dependent on the operating system."
///
/// In addition, we found the following undocumented limits:
///
///  - Width and height of 0 are invalid in all versions
///  - Widths and heights exceeding 0x666666 are invalid in all versions
///  - Pixel counts (of any width/height) exceeding 0x20000000 pixels
///
/// All of these are curently enforced.
pub fn is_size_valid(swf_version: u8, width: u32, height: u32) -> bool {
    // From :
    //
    // In addition, width and height of 0 are invalid in all versions.
    if width == 0 || height == 0 {
        return false;
    }
    if swf_version <= 9 {
        if width > 2880 || height > 2880 {
            return false;
        }
    } else if swf_version <= 12 {
        if width >= 0x2000 || height >= 0x2000 || width * height >= 0x1000000 {
            return false;
        }
    } else {
        // These limits are undocumented, but seem to be reliable.
        // TODO: Do they vary across different machines?
        if width > 0x6666666 || height > 0x6666666 || width as u64 * height as u64 >= 0x20000000 {
            return false;
        }
    }
    true
}
