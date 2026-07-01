This SWF was hand-crafted to test that data beyond the header-declared
uncompressed_len is not accessible, matching Flash Player behavior.

The SWF has two frames:
  Frame 1: trace("BEFORE")
  Frame 2: trace("AFTER")

The header's uncompressed_len is set to cut off right before frame 2.
Flash Player only executes frame 1, so only "BEFORE" should appear.
