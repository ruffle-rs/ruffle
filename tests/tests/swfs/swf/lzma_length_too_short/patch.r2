# ============================================================
# Set SWF length to a shorter value.
#
# The declared uncompressed length (the value written below) counts
# from the very start of the file, including the 8-byte ZWS header
# itself. 383 = 8 (that header) + 13 (RECT + frame rate + frame count)
# + 362 (FileAttributes + SetBackgroundColor + DoAction + ShowFrame
# tags), i.e. it points right after frame 1's ShowFrame tag. This
# keeps frame 1 intact but cuts off frame 2, similar to
# swf_length_too_short_no_second_frame.
#
# Unlike that test, this isn't expressed relative to the file size
# ($s): for an LZMA SWF, $s is the *compressed* size, which has no
# fixed relationship to the real uncompressed length.
#
# Apply with:
#   cp original.swf test.swf && r2 -w -q -i patch.r2 test.swf
# ============================================================

wv4 383 @ 4
