These tests are built with ASC with something like:

```
asc
-import playerglobal.abc
-in ../lib/print.as 
-in ../lib/com/adobe/test/Assert.as
-in ../lib/com/adobe/test/Utils.as
lf32/Test.as
```

Then combine the resulting `asc` file with `generated/intrinsics.asc` and stuff them both into a swf.

Example code:

```rust
let mut swf = vec![];
swf::write::write_swf(
    &swf::Header {
        compression: swf::Compression::None,
        version: 30,
        stage_size: swf::Rectangle {
            x_min: swf::Twips::ZERO,
            x_max: swf::Twips::from_pixels(100.0),
            y_min: swf::Twips::ZERO,
            y_max: swf::Twips::from_pixels(100.0),
        },
        frame_rate: swf::Fixed8::ONE,
        num_frames: 1,
    },
    &[
        swf::Tag::FileAttributes(swf::FileAttributes::IS_ACTION_SCRIPT_3),
        swf::Tag::DoAbc(swf::DoAbc {
            name: SwfStr::from_utf8_str("intrinsics"),
            is_lazy_initialize: false,
            data: intrinsics_abc,
        }),
        swf::Tag::DoAbc(swf::DoAbc {
            name: SwfStr::from_utf8_str(name),
            is_lazy_initialize: false,
            data: test_abc,
        }),
        swf::Tag::ShowFrame,
    ],
    &mut swf,
)?;
write(testdir.join("test.swf"), swf)?;
```