# SWF Regression Tests

Inside [tests/swfs](tests/swfs) is a large collection of automated tests that are based around running a swf and seeing what happens.

To create a test, make a directory that looks like the following, at minimum:

- directory/
  - test.swf
  - test.toml
  - output.txt

As best practice, please also include any source used to make the swf - such as `test.fla` and any actionscript files.


# Test Structure
## test.toml
Except for `num_ticks`, every other field and section is optional.

```toml
num_ticks = 1 # The amount of frames of the SWF to run.
tick_rate = 16.666 # The amount of time to process per tick. By default this uses the SWF frame rate.
sleep_to_meet_frame_rate = false # If true, sleep in between ticks to run at realtime speed. Necessary for some timer tests.
ignore = false # If true, ignore this test. Please comment why, ideally link to an issue, so we know what's up
known_failure = false # If true, this test is known to fail and the result will be inverted. When the test passes in the future, it'll fail and alert that it now passes.
output_path = "output.txt" # Path (relative to the directory containing test.toml) to the expected output
log_fetch = false # If true, all network requests will be included in the output.

# Sometimes floating point math doesn't exactly 100% match between flash and rust.
# If you encounter this in a test, the following section will change the output testing from "exact" to "approximate"
# (when it comes to floating point numbers, at least.)
[approximations]
number_patterns = [] # A list of regex patterns with capture groups to additionally treat as approximate numbers
epsilon = 0.0 # The upper bound of any rounding errors. Default is the difference between 1.0 and the next largest representable number
max_relative = 0.0 # The default relative tolerance for testing values that are far-apart. Default is the difference between 1.0 and the next largest representable number

# Options for the player used to run this swf
[player_options]
max_execution_duration = { secs = 15, nanos = 0} # How long can actionscript execute for before being forcefully stopped
viewport_dimensions = { width = 100, height = 100, scale_factor = 1 } # The size of the player. Defaults to the swfs stage size
with_renderer = { optional = false, sample_count = 4 } # If this test requires a renderer to run. Optional will enable the renderer where available.
with_audio = false # If this test requires an audio backend to run.
with_video = false # If this test requires a video decoder backend to run.
runtime = "AIR" # The runtime to emulate ("FlashPlayer" or "AIR"). Defaults to "FlashPlayer"

# A list of image comparisons to perform during the test. This block is repeatable infinitely, as long as each name is unique.
# The comparison part of a test is optional and only runs when `imgtests` feature is enabled
# This requires a render to be setup for this test
[image_comparisons.COMPARISON_NAME] # COMPARISON_NAME is a name of this particular image
tolerance = 0 # The tolerance per pixel channel to be considered "the same". Increase as needed with tests that aren't pixel perfect across platforms.
max_outliers = 0 # Maximum number of outliers allowed over the given tolerance levels. Increase as needed with tests that aren't pixel perfect across platforms.
trigger = "last_frame" # When to trigger this capture. Options are last_frame (default), fs_command, or a frame/tick number (1-based). Only one image may exist per frame/tick number or last_frame.

# Which build features are required for this test to run.
[required_features]
lzma = false # If LZMA support is enabled in this build
jpegxr = false # If JPEG XR support is enabled in this build
```

## Frame-based tests

Some older tests break with tick timing, so they instead use frames. When `num_frames` is specified, Ruffle's `tick` method will not be called and tick-based processing will not occur. Instead, `run_frame` will be called directly.

Tests that use video or other tick processing must not use `num_frames`, and in general its use is deprecated.

```toml
num_frames = 1 # The amount of frames of the swf to run
sleep_to_meet_frame_rate = false # If true, slow the tick rate to match the movies requested fps rate
```

## Quit on demand

`fscommand("quit")` is enabled for tests, and will end the test at the end of this frame or tick.

You can use this to end a test prematurely before the set number of iterations elapses, which may be useful for timer tests.