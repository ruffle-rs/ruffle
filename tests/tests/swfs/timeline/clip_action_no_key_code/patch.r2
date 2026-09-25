# ============================================================
# patch.r2 — patch "patch1" and "patch2" sites in an FWS SWF
#
# Per site: set clip action length to 0 and remove key code byte.
# Then fix the SWF FileLength field once, after both removals.
#
# Apply with:
#   cp original.swf test.swf
#   r2 -w -q -i patch.r2 test.swf
# ============================================================

# ---- Site 1: "patch1" -------------------------------------

/ patch1
s hit0_0
s +17
wx 00
r-1

# ---- Site 2: "patch2" -------------------------------------

/ patch2
s hit1_0
s +17
wx 00
r-1

# ---- Fix SWF FileLength ------------------------------------

wv4 `i~size[1]` @ 4
