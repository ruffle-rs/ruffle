use swf::avm2::types::*;
use swf::avm2::write::Writer as AbcWriter;
use swf::*;

fn main() {
    let abc = AbcFile {
        major_version: 46,
        minor_version: 16,
        constant_pool: ConstantPool {
            ints: vec![],
            uints: vec![],
            doubles: vec![],
            strings: vec![
                b"".to_vec(),                                                         // 1
                b"<root><item>hello</item><item>world</item></root>".to_vec(),        // 2
                b"trace".to_vec(),                                                    // 3
                b"XML".to_vec(),                                                      // 4
                b"name".to_vec(),                                                     // 5
                b"name() via with-scope: ".to_vec(),                                  // 6
                b"localName".to_vec(),                                                // 7
                b"localName() via with-scope: ".to_vec(),                             // 8
                b"nodeKind".to_vec(),                                                 // 9
                b"nodeKind() via with-scope: ".to_vec(),                              // 10
                b"children".to_vec(),                                                 // 11
                b"length".to_vec(),                                                   // 12
                b"children().length() via with-scope: ".to_vec(),                     // 13
                b"toString".to_vec(),                                                 // 14
                b"toString() via with-scope: ".to_vec(),                              // 15
            ],
            namespaces: vec![
                Namespace::Package(Index::new(1)),  // 1: public (string idx 1 = "")
            ],
            namespace_sets: vec![],
            multinames: vec![
                // 1: public::trace
                Multiname::QName { namespace: Index::new(1), name: Index::new(3) },
                // 2: public::XML
                Multiname::QName { namespace: Index::new(1), name: Index::new(4) },
                // 3: public::name  (PUBLIC namespace, not AS3 - this is the key!)
                Multiname::QName { namespace: Index::new(1), name: Index::new(5) },
                // 4: public::localName
                Multiname::QName { namespace: Index::new(1), name: Index::new(7) },
                // 5: public::nodeKind
                Multiname::QName { namespace: Index::new(1), name: Index::new(9) },
                // 6: public::children
                Multiname::QName { namespace: Index::new(1), name: Index::new(11) },
                // 7: public::length
                Multiname::QName { namespace: Index::new(1), name: Index::new(12) },
                // 8: public::toString
                Multiname::QName { namespace: Index::new(1), name: Index::new(14) },
            ],
        },
        methods: vec![
            Method {
                name: Index::new(0),
                params: vec![],
                return_type: Index::new(0),
                flags: MethodFlags::empty(),
                body: Some(Index::new(0)),
            },
        ],
        metadata: vec![],
        instances: vec![],
        classes: vec![],
        scripts: vec![
            Script {
                init_method: Index::new(0),
                traits: vec![],
            },
        ],
        method_bodies: vec![
            MethodBody {
                method: Index::new(0),
                max_stack: 6,
                num_locals: 2,
                init_scope_depth: 1,
                max_scope_depth: 3,
                code: vec![],  // will be filled below
                exceptions: vec![],
                traits: vec![],
            },
        ],
    };

    // Build bytecode using the avm2 writer
    let mut code_buf = Vec::new();
    {
        let mut w = AbcWriter::new(&mut code_buf);

        w.write_op(&Op::GetLocal { index: 0 }).unwrap();
        w.write_op(&Op::PushScope).unwrap();

        // var xml = new XML("<root>...");
        w.write_op(&Op::FindPropStrict { index: Index::new(2) }).unwrap();
        w.write_op(&Op::PushString { value: Index::new(2) }).unwrap();
        w.write_op(&Op::ConstructProp { index: Index::new(2), num_args: 1 }).unwrap();
        w.write_op(&Op::CoerceA).unwrap();
        w.write_op(&Op::SetLocal { index: 1 }).unwrap();

        // with(xml) {
        w.write_op(&Op::GetLocal { index: 1 }).unwrap();
        w.write_op(&Op::PushWith).unwrap();

        // Helper to emit: trace(label + method())
        fn emit_trace(w: &mut AbcWriter<&mut Vec<u8>>, label: u32, method: u32) {
            w.write_op(&Op::FindPropStrict { index: Index::new(1) }).unwrap();
            w.write_op(&Op::PushString { value: Index::new(label) }).unwrap();
            w.write_op(&Op::FindPropStrict { index: Index::new(method) }).unwrap();
            w.write_op(&Op::CallProperty { index: Index::new(method), num_args: 0 }).unwrap();
            w.write_op(&Op::CoerceS).unwrap();
            w.write_op(&Op::Add).unwrap();
            w.write_op(&Op::CallPropVoid { index: Index::new(1), num_args: 1 }).unwrap();
        }

        emit_trace(&mut w, 6, 3);   // name()
        emit_trace(&mut w, 8, 4);    // localName()
        emit_trace(&mut w, 10, 5);   // nodeKind()
        emit_trace(&mut w, 15, 8);   // toString()

        // children().length()
        w.write_op(&Op::FindPropStrict { index: Index::new(1) }).unwrap();
        w.write_op(&Op::PushString { value: Index::new(13) }).unwrap();
        w.write_op(&Op::FindPropStrict { index: Index::new(6) }).unwrap();
        w.write_op(&Op::CallProperty { index: Index::new(6), num_args: 0 }).unwrap();
        w.write_op(&Op::CallProperty { index: Index::new(7), num_args: 0 }).unwrap();
        w.write_op(&Op::CoerceS).unwrap();
        w.write_op(&Op::Add).unwrap();
        w.write_op(&Op::CallPropVoid { index: Index::new(1), num_args: 1 }).unwrap();

        // } end with
        w.write_op(&Op::PopScope).unwrap();
        w.write_op(&Op::ReturnVoid).unwrap();
    }

    // Now write the full ABC with the code
    let mut abc_with_code = abc;
    abc_with_code.method_bodies[0].code = code_buf;

    let mut abc_bytes = Vec::new();
    AbcWriter::new(&mut abc_bytes).write(abc_with_code).unwrap();

    // Build SWF
    let header = Header {
        compression: Compression::Zlib,
        version: 11,
        stage_size: Rectangle {
            x_min: Twips::ZERO,
            x_max: Twips::from_pixels(550.0),
            y_min: Twips::ZERO,
            y_max: Twips::from_pixels(400.0),
        },
        frame_rate: Fixed8::from_f32(24.0),
        num_frames: 1,
    };

    let tags = vec![
        Tag::FileAttributes(FileAttributes::IS_ACTION_SCRIPT_3),
        Tag::DoAbc2(DoAbc2 {
            flags: DoAbc2Flag::empty(),
            name: SwfStr::from_utf8_str("test"),
            data: &abc_bytes,
        }),
        Tag::ShowFrame,
    ];

    let mut swf_bytes = Vec::new();
    swf::write::write_swf(&header, &tags, &mut swf_bytes).unwrap();

    std::fs::write("/tmp/test_with_scope.swf", &swf_bytes).unwrap();
    println!("Generated SWF: {} bytes", swf_bytes.len());
}
