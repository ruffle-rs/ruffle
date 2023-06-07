use crate::pixel_bender::{
    Opcode, Operation, PixelBenderMetadata, PixelBenderParam, PixelBenderParamQualifier,
    PixelBenderReg, PixelBenderRegChannel, PixelBenderRegKind, PixelBenderShader, PixelBenderType,
    PixelBenderTypeOpcode,
};

use super::parse_shader;

#[test]
fn simple_shader() {
    let shader = &[
        165, 1, 0, 0, 0, 164, 9, 0, 68, 111, 78, 111, 116, 104, 105, 110, 103, 160, 12, 110, 97,
        109, 101, 115, 112, 97, 99, 101, 0, 65, 100, 111, 98, 101, 58, 58, 69, 120, 97, 109, 112,
        108, 101, 0, 160, 12, 118, 101, 110, 100, 111, 114, 0, 65, 100, 111, 98, 101, 32, 101, 120,
        97, 109, 112, 108, 101, 115, 0, 160, 8, 118, 101, 114, 115, 105, 111, 110, 0, 1, 0, 160,
        12, 100, 101, 115, 99, 114, 105, 112, 116, 105, 111, 110, 0, 65, 32, 115, 104, 97, 100,
        101, 114, 32, 116, 104, 97, 116, 32, 100, 111, 101, 115, 32, 110, 111, 116, 104, 105, 110,
        103, 44, 32, 98, 117, 116, 32, 100, 111, 101, 115, 32, 105, 116, 32, 119, 101, 108, 108,
        46, 0, 161, 1, 2, 0, 0, 12, 95, 79, 117, 116, 67, 111, 111, 114, 100, 0, 163, 0, 4, 115,
        114, 99, 0, 161, 2, 4, 1, 0, 15, 100, 115, 116, 0, 161, 1, 2, 0, 0, 3, 115, 105, 122, 101,
        0, 162, 12, 100, 101, 115, 99, 114, 105, 112, 116, 105, 111, 110, 0, 84, 104, 101, 32, 115,
        105, 122, 101, 32, 111, 102, 32, 116, 104, 101, 32, 105, 109, 97, 103, 101, 32, 116, 111,
        32, 119, 104, 105, 99, 104, 32, 116, 104, 101, 32, 107, 101, 114, 110, 101, 108, 32, 105,
        115, 32, 97, 112, 112, 108, 105, 101, 100, 0, 162, 2, 109, 105, 110, 86, 97, 108, 117, 101,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 162, 2, 109, 97, 120, 86, 97, 108, 117, 101, 0, 66, 200, 0, 0,
        66, 200, 0, 0, 162, 2, 100, 101, 102, 97, 117, 108, 116, 86, 97, 108, 117, 101, 0, 66, 72,
        0, 0, 66, 72, 0, 0, 161, 1, 1, 2, 0, 8, 114, 97, 100, 105, 117, 115, 0, 162, 12, 100, 101,
        115, 99, 114, 105, 112, 116, 105, 111, 110, 0, 84, 104, 101, 32, 114, 97, 100, 105, 117,
        115, 32, 111, 102, 32, 116, 104, 101, 32, 101, 102, 102, 101, 99, 116, 0, 162, 1, 109, 105,
        110, 86, 97, 108, 117, 101, 0, 0, 0, 0, 0, 162, 1, 109, 97, 120, 86, 97, 108, 117, 101, 0,
        66, 72, 0, 0, 162, 1, 100, 101, 102, 97, 117, 108, 116, 86, 97, 108, 117, 101, 0, 65, 200,
        0, 0, 4, 2, 0, 64, 2, 0, 0, 0, 3, 2, 0, 64, 2, 0, 0, 0, 4, 2, 0, 49, 0, 0, 176, 0, 3, 2, 0,
        49, 0, 0, 176, 0, 29, 3, 0, 193, 2, 0, 80, 0, 3, 3, 0, 193, 2, 0, 176, 0, 29, 2, 0, 97, 3,
        0, 16, 0, 48, 3, 0, 241, 0, 0, 16, 0, 50, 4, 0, 128, 66, 200, 0, 0, 50, 4, 0, 64, 0, 0, 0,
        0, 50, 4, 0, 32, 66, 200, 0, 0, 50, 4, 0, 16, 63, 128, 0, 0, 29, 5, 0, 243, 3, 0, 27, 0, 1,
        5, 0, 243, 4, 0, 27, 0, 29, 1, 0, 243, 5, 0, 27, 0,
    ];

    let expected = PixelBenderShader {
        name: "DoNothing".to_string(),
        version: 1,
        params: vec![
            PixelBenderParam::Normal {
                qualifier: PixelBenderParamQualifier::Input,
                param_type: PixelBenderTypeOpcode::TFloat2,
                reg: PixelBenderReg {
                    index: 0,
                    channels: vec![PixelBenderRegChannel::R, PixelBenderRegChannel::G],
                    kind: PixelBenderRegKind::Float,
                },
                name: "_OutCoord".to_string(),
                metadata: vec![],
            },
            PixelBenderParam::Texture {
                index: 0,
                channels: 4,
                name: "src".to_string(),
            },
            PixelBenderParam::Normal {
                qualifier: PixelBenderParamQualifier::Output,
                param_type: PixelBenderTypeOpcode::TFloat4,
                reg: PixelBenderReg {
                    index: 1,
                    channels: PixelBenderRegChannel::RGBA.to_vec(),
                    kind: PixelBenderRegKind::Float,
                },
                name: "dst".to_string(),
                metadata: vec![],
            },
            PixelBenderParam::Normal {
                qualifier: PixelBenderParamQualifier::Input,
                param_type: PixelBenderTypeOpcode::TFloat2,
                reg: PixelBenderReg {
                    index: 0,
                    channels: vec![PixelBenderRegChannel::B, PixelBenderRegChannel::A],
                    kind: PixelBenderRegKind::Float,
                },
                name: "size".to_string(),
                metadata: vec![
                    PixelBenderMetadata {
                        key: "description".to_string(),
                        value: PixelBenderType::TString(
                            "The size of the image to which the kernel is applied".to_string(),
                        ),
                    },
                    PixelBenderMetadata {
                        key: "minValue".to_string(),
                        value: PixelBenderType::TFloat2(0.0, 0.0),
                    },
                    PixelBenderMetadata {
                        key: "maxValue".to_string(),
                        value: PixelBenderType::TFloat2(100.0, 100.0),
                    },
                    PixelBenderMetadata {
                        key: "defaultValue".to_string(),
                        value: PixelBenderType::TFloat2(50.0, 50.0),
                    },
                ],
            },
            PixelBenderParam::Normal {
                qualifier: PixelBenderParamQualifier::Input,
                param_type: PixelBenderTypeOpcode::TFloat,
                reg: PixelBenderReg {
                    index: 2,
                    channels: vec![PixelBenderRegChannel::R],
                    kind: PixelBenderRegKind::Float,
                },
                name: "radius".to_string(),
                metadata: vec![
                    PixelBenderMetadata {
                        key: "description".to_string(),
                        value: PixelBenderType::TString("The radius of the effect".to_string()),
                    },
                    PixelBenderMetadata {
                        key: "minValue".to_string(),
                        value: PixelBenderType::TFloat(0.0),
                    },
                    PixelBenderMetadata {
                        key: "maxValue".to_string(),
                        value: PixelBenderType::TFloat(50.0),
                    },
                    PixelBenderMetadata {
                        key: "defaultValue".to_string(),
                        value: PixelBenderType::TFloat(25.0),
                    },
                ],
            },
        ],
        metadata: vec![
            PixelBenderMetadata {
                key: "namespace".to_string(),
                value: PixelBenderType::TString("Adobe::Example".to_string()),
            },
            PixelBenderMetadata {
                key: "vendor".to_string(),
                value: PixelBenderType::TString("Adobe examples".to_string()),
            },
            PixelBenderMetadata {
                key: "version".to_string(),
                value: PixelBenderType::TInt(1),
            },
            PixelBenderMetadata {
                key: "description".to_string(),
                value: PixelBenderType::TString(
                    "A shader that does nothing, but does it well.".to_string(),
                ),
            },
        ],
        operations: vec![
            Operation::Normal {
                opcode: Opcode::Rcp,
                dst: PixelBenderReg {
                    index: 2,
                    channels: vec![PixelBenderRegChannel::G],
                    kind: PixelBenderRegKind::Float,
                },
                src: PixelBenderReg {
                    index: 2,
                    channels: vec![PixelBenderRegChannel::R],
                    kind: PixelBenderRegKind::Float,
                },
            },
            Operation::Normal {
                opcode: Opcode::Mul,
                dst: PixelBenderReg {
                    index: 2,
                    channels: vec![PixelBenderRegChannel::G],
                    kind: PixelBenderRegKind::Float,
                },
                src: PixelBenderReg {
                    index: 2,
                    channels: vec![PixelBenderRegChannel::R],
                    kind: PixelBenderRegKind::Float,
                },
            },
            Operation::Normal {
                opcode: Opcode::Rcp,
                dst: PixelBenderReg {
                    index: 2,
                    channels: vec![PixelBenderRegChannel::B, PixelBenderRegChannel::A],
                    kind: PixelBenderRegKind::Float,
                },
                src: PixelBenderReg {
                    index: 0,
                    channels: vec![PixelBenderRegChannel::B, PixelBenderRegChannel::A],
                    kind: PixelBenderRegKind::Float,
                },
            },
            Operation::Normal {
                opcode: Opcode::Mul,
                dst: PixelBenderReg {
                    index: 2,
                    channels: vec![PixelBenderRegChannel::B, PixelBenderRegChannel::A],
                    kind: PixelBenderRegKind::Float,
                },
                src: PixelBenderReg {
                    index: 0,
                    channels: vec![PixelBenderRegChannel::B, PixelBenderRegChannel::A],
                    kind: PixelBenderRegKind::Float,
                },
            },
            Operation::Normal {
                opcode: Opcode::Mov,
                dst: PixelBenderReg {
                    index: 3,
                    channels: vec![PixelBenderRegChannel::R, PixelBenderRegChannel::G],
                    kind: PixelBenderRegKind::Float,
                },
                src: PixelBenderReg {
                    index: 2,
                    channels: vec![PixelBenderRegChannel::G, PixelBenderRegChannel::G],
                    kind: PixelBenderRegKind::Float,
                },
            },
            Operation::Normal {
                opcode: Opcode::Mul,
                dst: PixelBenderReg {
                    index: 3,
                    channels: vec![PixelBenderRegChannel::R, PixelBenderRegChannel::G],
                    kind: PixelBenderRegKind::Float,
                },
                src: PixelBenderReg {
                    index: 2,
                    channels: vec![PixelBenderRegChannel::B, PixelBenderRegChannel::A],
                    kind: PixelBenderRegKind::Float,
                },
            },
            Operation::Normal {
                opcode: Opcode::Mov,
                dst: PixelBenderReg {
                    index: 2,
                    channels: vec![PixelBenderRegChannel::G, PixelBenderRegChannel::B],
                    kind: PixelBenderRegKind::Float,
                },
                src: PixelBenderReg {
                    index: 3,
                    channels: vec![PixelBenderRegChannel::R, PixelBenderRegChannel::G],
                    kind: PixelBenderRegKind::Float,
                },
            },
            Operation::SampleNearest {
                dst: PixelBenderReg {
                    index: 3,
                    channels: PixelBenderRegChannel::RGBA.to_vec(),
                    kind: PixelBenderRegKind::Float,
                },
                src: PixelBenderReg {
                    index: 0,
                    channels: vec![PixelBenderRegChannel::R, PixelBenderRegChannel::G],
                    kind: PixelBenderRegKind::Float,
                },
                tf: 0,
            },
            Operation::LoadFloat {
                dst: PixelBenderReg {
                    index: 4,
                    channels: vec![PixelBenderRegChannel::R],
                    kind: PixelBenderRegKind::Float,
                },
                val: 100.0,
            },
            Operation::LoadFloat {
                dst: PixelBenderReg {
                    index: 4,
                    channels: vec![PixelBenderRegChannel::G],
                    kind: PixelBenderRegKind::Float,
                },
                val: 0.0,
            },
            Operation::LoadFloat {
                dst: PixelBenderReg {
                    index: 4,
                    channels: vec![PixelBenderRegChannel::B],
                    kind: PixelBenderRegKind::Float,
                },
                val: 100.0,
            },
            Operation::LoadFloat {
                dst: PixelBenderReg {
                    index: 4,
                    channels: vec![PixelBenderRegChannel::A],
                    kind: PixelBenderRegKind::Float,
                },
                val: 1.0,
            },
            Operation::Normal {
                opcode: Opcode::Mov,
                dst: PixelBenderReg {
                    index: 5,
                    channels: vec![
                        PixelBenderRegChannel::R,
                        PixelBenderRegChannel::G,
                        PixelBenderRegChannel::B,
                        PixelBenderRegChannel::A,
                    ],
                    kind: PixelBenderRegKind::Float,
                },
                src: PixelBenderReg {
                    index: 3,
                    channels: vec![
                        PixelBenderRegChannel::R,
                        PixelBenderRegChannel::G,
                        PixelBenderRegChannel::B,
                        PixelBenderRegChannel::A,
                    ],
                    kind: PixelBenderRegKind::Float,
                },
            },
            Operation::Normal {
                opcode: Opcode::Add,
                dst: PixelBenderReg {
                    index: 5,
                    channels: vec![
                        PixelBenderRegChannel::R,
                        PixelBenderRegChannel::G,
                        PixelBenderRegChannel::B,
                        PixelBenderRegChannel::A,
                    ],
                    kind: PixelBenderRegKind::Float,
                },
                src: PixelBenderReg {
                    index: 4,
                    channels: PixelBenderRegChannel::RGBA.to_vec(),
                    kind: PixelBenderRegKind::Float,
                },
            },
            Operation::Normal {
                opcode: Opcode::Mov,
                dst: PixelBenderReg {
                    index: 1,
                    channels: PixelBenderRegChannel::RGBA.to_vec(),
                    kind: PixelBenderRegKind::Float,
                },
                src: PixelBenderReg {
                    index: 5,
                    channels: PixelBenderRegChannel::RGBA.to_vec(),
                    kind: PixelBenderRegKind::Float,
                },
            },
        ],
    };

    let shader = parse_shader(shader).expect("Failed to parse shader");
    assert_eq!(shader, expected, "Shader parsed incorrectly!");
}
