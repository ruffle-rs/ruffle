use swf::{Compression, Fixed8, Rectangle, Twips};

// These are macros instead of functions, so that failures point to the test.

/// Compiles the given tags and returns the serialized tag stream.
macro_rules! compile_tags {
    ($tags:expr) => {{
        let compile = |tags: &str| {
            swfasm::compile_str(&format!("Swf(version: 10, num_frames: 0, tags: [{tags}])"))
        };
        let header_len = compile("").unwrap().len();
        compile(&$tags).unwrap()[header_len..].to_vec()
    }};
}

/// Compiles the given source, which must fail, and returns the error message.
macro_rules! compile_err {
    ($source:expr) => {
        format!("{:#}", swfasm::compile_str(&$source).unwrap_err())
    };
}

#[test]
fn header() {
    let swf = swfasm::compile_str(
        "Swf(
            version: 8,
            stage: (x: 10, y: 1.5, width: 100, height: 0.05),
            frame_rate: 12.5,
            tags: [ShowFrame, ShowFrame, ShowFrame],
        )",
    )
    .unwrap();
    let swf_buf = swf::decompress_swf(&swf[..]).unwrap();
    let header = &swf_buf.header;
    assert_eq!(header.compression(), Compression::None);
    assert_eq!(header.version(), 8);
    assert_eq!(
        header.stage_size(),
        &Rectangle {
            x_min: Twips::new(200),
            x_max: Twips::new(2200),
            y_min: Twips::new(30),
            y_max: Twips::new(31),
        }
    );
    assert_eq!(header.frame_rate(), Fixed8::from_f64(12.5));
    assert_eq!(header.num_frames(), 3);
}

#[test]
fn stage_edges_are_added_in_twips() {
    let swf =
        swfasm::compile_str("Swf(version: 10, stage: (x: 0.05, y: 0.05, width: 0.1, height: 0.1))")
            .unwrap();
    let swf_buf = swf::decompress_swf(&swf[..]).unwrap();
    assert_eq!(
        swf_buf.header.stage_size(),
        &Rectangle {
            x_min: Twips::new(1),
            x_max: Twips::new(3),
            y_min: Twips::new(1),
            y_max: Twips::new(3),
        }
    );
}

#[test]
fn default_num_frames() {
    let num_frames = |tags: &str| {
        let swf = swfasm::compile_str(&format!("Swf(version: 10, tags: [{tags}])")).unwrap();
        swf::decompress_swf(&swf[..]).unwrap().header.num_frames()
    };
    assert_eq!(num_frames(r#"Raw(code: 1, data: "")"#), 1);
}

#[test]
fn file_attributes_flags() {
    assert_eq!(
        compile_tags!("FileAttributes(as3: true, use_network_sandbox: true)"),
        [0x44, 0x11, 0x09, 0x00, 0x00, 0x00]
    );
}

#[test]
fn raw_tag() {
    assert_eq!(
        compile_tags!(r#"Raw(code: 1023, data: "01 02 aB"), Raw(code: 2, data: "0304")"#),
        [0xC3, 0xFF, 0x01, 0x02, 0xAB, 0x82, 0x00, 0x03, 0x04]
    );
    assert_eq!(
        compile_tags!(
            r#"Raw(code: 2, data: [
                "05 06", // comment
                "",
                " 07 ",
            ])"#
        ),
        [0x83, 0x00, 0x05, 0x06, 0x07]
    );
    assert_eq!(compile_tags!(r#"Raw(code: 2, data: [])"#), [0x80, 0x00]);
    assert_eq!(
        compile_tags!(r#"Raw(code: 2, force_long: true, data: "aa")"#),
        [0xBF, 0x00, 0x01, 0x00, 0x00, 0x00, 0xAA]
    );
    // Bodies that do not fit in the short form use the long one anyway.
    assert_eq!(
        compile_tags!(&format!(r#"Raw(code: 2, data: "{}")"#, "00".repeat(63))),
        [[0xBF, 0x00, 63, 0x00, 0x00, 0x00].as_slice(), &[0; 63]].concat()
    );
}

#[test]
fn tag_code_errors() {
    assert!(
        compile_err!(r#"Swf(version: 10, tags: [Raw(code: 1100, data: "")])"#)
            .contains("does not fit in 10 bits")
    );
}

#[test]
fn unknown_fields() {
    assert!(compile_err!("Swf(version: 10, tags: [FileAttributes(as_3: true)])").contains("as_3"));
    assert!(
        compile_err!(r#"Swf(version: 10, tags: [Raw(code: 1, data: "", long: true)])"#)
            .contains("long")
    );
}

#[test]
fn do_action_bytes() {
    assert_eq!(
        compile_tags!(r#"DoAction(Bytes("07 00"))"#),
        [0x02, 0x03, 0x07, 0x00]
    );
}

#[test]
fn invalid_values() {
    assert!(
        compile_err!("Swf(version: 10, stage: (width: 0.01, height: 1))")
            .contains("not a whole number of twips")
    );
    assert!(
        compile_err!("Swf(version: 10, frame_rate: 0.001)")
            .contains("not representable as an 8.8 fixed point number")
    );
    assert!(
        compile_err!("Swf(version: 10, tags: [SetBackgroundColor(0x1000000)])")
            .contains("not in the 0xRRGGBB format")
    );
    assert!(compile_err!(r#"Swf(version: 10, tags: [Raw(code: 1, data: "0")])"#).contains("odd"));
    assert!(
        compile_err!(r#"Swf(version: 10, tags: [Raw(code: 1, data: ["00", "0"])])"#)
            .contains("odd")
    );
    assert!(
        compile_err!(r#"Swf(version: 10, tags: [Raw(code: 1, data: "0g")])"#)
            .contains("invalid hex digit 'g'")
    );
    assert!(
        compile_err!(r#"Swf(version: 10, tags: [Raw(code: 1, data: [1, 2])])"#).contains("string")
    );
}
