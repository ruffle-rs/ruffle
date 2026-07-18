# ============================================================
# patch.r2 — patch scene count in an FWS SWF
#
# Set scene count to a very large number.
# Then fix the SWF FileLength field.
#
# Apply with:
#   cp original.swf test.swf
#   r2 -w -q -i patch.r2 test.swf
# ============================================================

/x deadbeef
s hit0_0
s +6

r+4
wx f0ffffff0f

# ---- Fix SWF FileLength ------------------------------------

wv4 `i~size[1]` @ 4
