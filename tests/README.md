# SWF Regression Tests

Inside [tests/swfs](tests/swfs) is a large collection of automated tests that
are based around running a SWF and seeing what happens.

To create a test, make a directory that looks like the following, at minimum:

- `directory/`
    - `test.swf`
    - `test.toml`
    - `output.txt`

As best practice, please also include any source used to make the SWF, such as `test.fla` and/or any ActionScript files.

# Test Structure

## test.toml

Except for `num_ticks`, every other field and section is optional.

```toml
# The number of frames of the SWF to run.
num_ticks = 1

# The number of milliseconds to process per tick.
# By default this uses the SWF frame rate.
tick_rate = 16.666

# If true, sleep in between ticks to run at realtime speed.
# Necessary for some timer tests.
sleep_to_meet_frame_rate = false

# If true, ignore this test.
# Please comment why, ideally link to an issue, so we know what's up.
# Prefer setting `known_failure = true` to ignoring the test.
ignore = false

# If true, this test is known to fail and the test runner will expect the check against
# the trace output (specified `output_path`) to fail.
# When the test passes in the future, it'll fail and alert that it now passes.
# This will not catch Ruffle panics; if the test is expected to panic, use
#   `known_failure.panic = "panic message"`
# instead (note that 'panicky' tests will be skipped if the test harness is run
# with debug assertions disabled, e.g. with `--release`).
# By default, the test runner will additionally check Ruffle's output against itself
# to detect regressions in failing tests; this can be disabled with
#    `known_failure.ruffle_check = false`.
known_failure = false

# Path (relative to the directory containing test.toml) to the expected output
output_path = "output.txt"

# If true, all network requests will be included in the output.
log_fetch = false

# Sometimes floating point math doesn't exactly 100% match between Flash and Rust.
# If you encounter this in a test, the following section will change the output
# testing from "exact" to "approximate" (when it comes to floating point numbers, at least.)
[approximations]

# A list of regex patterns with capture groups to additionally treat as approximate numbers.
number_patterns = []

# The upper bound of any rounding errors.
# Default is the difference between 1.0 and the next largest representable number.
epsilon = 0.0

# The default relative tolerance for testing values that are far-apart.
# Default is the difference between 1.0 and the next largest representable number
max_relative = 0.0

# Options for the player used to run this SWF.
[player_options]

# How long can ActionScript execute for before being forcefully stopped.
max_execution_duration = { secs = 15, nanos = 0 }

# The size of the player. Defaults to the SWF's stage size.
viewport_dimensions = { width = 100, height = 100, scale_factor = 1 }

# If this test requires a renderer to run.
# Optional will run the test without the renderer when it's unavailable
# and will skip comparing visual outputs.
with_renderer = { optional = false, sample_count = 4 }

# If this test requires an audio backend to run.
with_audio = false

# If this test requires a video decoder backend to run.
with_video = false

# The runtime to emulate ("FlashPlayer" or "AIR"). Defaults to "FlashPlayer".
runtime = "AIR"

# The version of the player to emulate. If not set, it uses the newest one ruffle knows about.
version = 32

# Whether Ruffle's default font should be available.
# It's not recommended to enable this option, as it will introduce differences
# in behavior between Ruffle and Flash.
with_default_font = false

# A list of image comparisons to perform during the test. This block is repeatable infinitely, as long as each name is unique.
# The comparison part of a test is optional and only runs when `imgtests` feature is enabled
# This requires a render to be setup for this test
[image_comparisons.COMPARISON_NAME] # COMPARISON_NAME is a name of this particular image

# If true, this image comparison is known to fail and the test runner will expect it to fail.
# When the comparison passes in the future, it'll fail and alert that it now passes.
known_failure = false

# The tolerance per pixel channel to be considered "the same".
# Increase as needed with tests that aren't pixel perfect across platforms.
# Prefer running tests with higher sample count to make a better use of this option.
tolerance = 0

# Maximum number of outliers (pixel channel) allowed over the given tolerance levels.
# Increase as needed with tests that aren't pixel perfect across platforms.
max_outliers = 0

# When to trigger this capture.
# Options are last_frame (default), fs_command, or a frame/tick number (1-based).
# Only one image may exist per frame/tick number or last_frame.
trigger = "last_frame"

# A list of checks to perform during image comparison.
# They can be used instead of providing a single check in `image_comparisons.COMPARISON_NAME`.
# Every applicable check is being executed for each image.
[[image_comparisons.COMPARISON_NAME.checks]] # COMPARISON_NAME is a name of this particular image

# Same as `image_comparisons.COMPARISON_NAME.tolerance`, but for this particular check.
tolerance = 0

# Same as `image_comparisons.COMPARISON_NAME.max_outliers`, but for this particular check.
max_outliers = 0

# Filter is a cfg-like expression that checks if this particular check should be performed.
# It can be used to add different checks for e.g. different platforms.
#
# Available predicates:
#  * os
#  * arch
#  * family
filter = 'arch = "aarch64"'

# Which build features are required for this test to run.
[required_features]

# If LZMA support is enabled in this build
lzma = false

# If JPEG XR support is enabled in this build
jpegxr = false

# A single device font provided for this test.
[fonts.FONT_NAME] # FONT_NAME is a name of this particular font

# Font family, name of the font used when looking up fonts.
family = "Test Font"

# Path to the file containing the font.
path = "font_file.ttf"

# Whether the font should be considered as bold.
bold = true

# Whether the font should be considered as italic.
italic = true

# A single device font sort provided for this test.
# Font sort defines a list of fonts that should be used for a particular query.
[font_sorts.FONT_SORT_NAME] # FONT_SORT_NAME is a name of this particular font sort

# Font family, name of the queried font.
family = "Test Font"

# Whether the queried font is bold.
bold = true

# Whether the queried font is italic.
italic = true

# Sorted list of fonts returned for this particular query.
sort = ["Test Font 1", "Test Font 2"]

# Lists of device fonts defined for Flash's default fonts.
[default_fonts]
sans = ["Test Font", "Test Font Fallback"]
serif = ["Test Font", "Test Font Fallback"]
typewriter = ["Test Font", "Test Font Fallback"]
japanese_gothic = ["Test Font", "Test Font Fallback"]
japanese_gothic_mono = ["Test Font", "Test Font Fallback"]
japanese_mincho = ["Test Font", "Test Font Fallback"]

# Support for multiple configurations (see the 'Multiple tests' section for more details)
# Fields specified here will override the 'default' values provided in the rest of the document.
[subtests.SUBTEST_NAME]
# ...
```

## Multiple tests

Sometimes, you may want to test a given `test.swf` several times with different settings.
For this situation, a `[subtests]` section can be added for each distinct configuration to be tested;
each configuration will be ran as a separate test.

For example, if `test.swf` has different output depending on the Flash Player version, the following
`test.toml` could be used:
```toml
num_ticks = 1
# other common settings...

[subtests.fp9]
output_path = "output.fp9.txt"
player_options.version = 9

[subtests.fp10]
output_path = "output.fp10.txt"
player_options.version = 10
```

## Frame-based tests

Some older tests break with tick timing, so they instead use frames.
When `num_frames` is specified, Ruffle's `tick` method will not be called and tick-based processing will not occur.
Instead, `run_frame` will be called directly.

Tests that use video or other tick processing must not use `num_frames`, and in general its use is deprecated.

```toml
# The number of frames of the SWF to run.
num_frames = 1

# If true, slow the tick rate to match the movie's requested FPS rate.
sleep_to_meet_frame_rate = false
```

## Quit on demand

`fscommand("quit")` is enabled for tests, and will end the test at the end of this frame or tick.

You can use this to end a test prematurely before the set number of iterations elapses, which may be useful for timer
tests.
