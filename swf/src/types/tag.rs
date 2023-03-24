use crate::types::*;

/// Represents a tag in an SWF file.
///
/// The SWF format is made up of a stream of tags. Each tag either
/// defines a character (Graphic, Sound, MovieClip), or places/modifies
/// an instance of these characters on the display list.
///
// [SWF19 p.29](https://www.adobe.com/content/dam/acom/en/devnet/pdf/swf-file-format-spec.pdf#page=29)
#[derive(Debug, PartialEq)]
pub enum Tag<'a> {
    CsmTextSettings(CsmTextSettings),
    DebugId(DebugId),
    DefineBinaryData(DefineBinaryData<'a>),
    DefineBits {
        id: CharacterId,
        jpeg_data: &'a [u8],
    },
    DefineBitsJpeg2 {
        id: CharacterId,
        jpeg_data: &'a [u8],
    },
    DefineBitsJpeg3(DefineBitsJpeg3<'a>),
    DefineBitsLossless(DefineBitsLossless<'a>),
    DefineButton(Box<Button<'a>>),
    DefineButton2(Box<Button<'a>>),
    DefineButtonColorTransform(ButtonColorTransform),
    DefineButtonSound(Box<ButtonSounds>),
    DefineEditText(Box<EditText<'a>>),
    DefineFont(Box<FontV1>),
    DefineFont2(Box<Font<'a>>),
    DefineFont4(Font4<'a>),
    DefineFontAlignZones {
        id: CharacterId,
        thickness: FontThickness,
        zones: Vec<FontAlignZone>,
    },
    DefineFontInfo(Box<FontInfo<'a>>),
    DefineFontName {
        id: CharacterId,
        name: &'a SwfStr,
        copyright_info: &'a SwfStr,
    },
    DefineMorphShape(Box<DefineMorphShape>),
    DefineScalingGrid {
        id: CharacterId,
        splitter_rect: Rectangle<Twips>,
    },
    DefineSceneAndFrameLabelData(DefineSceneAndFrameLabelData<'a>),
    DefineShape(Shape),
    DefineSound(Box<Sound<'a>>),
    DefineSprite(Sprite<'a>),
    DefineText(Box<Text>),
    DefineVideoStream(DefineVideoStream),
    DoAbc(&'a [u8]),
    DoAbc2(DoAbc2<'a>),
    DoAction(DoAction<'a>),
    DoInitAction {
        id: CharacterId,
        action_data: &'a [u8],
    },
    EnableDebugger(&'a SwfStr),
    EnableTelemetry {
        password_hash: &'a [u8],
    },
    End,
    ExportAssets(ExportAssets<'a>),
    FileAttributes(FileAttributes),
    FrameLabel(FrameLabel<'a>),
    ImportAssets {
        url: &'a SwfStr,
        imports: Vec<ExportedAsset<'a>>,
    },
    JpegTables(JpegTables<'a>),
    Metadata(&'a SwfStr),
    NameCharacter(NameCharacter<'a>),
    RemoveObject(RemoveObject),
    PlaceObject(Box<PlaceObject<'a>>),
    ProductInfo(ProductInfo),
    Protect(Option<&'a SwfStr>),
    ScriptLimits {
        max_recursion_depth: u16,
        timeout_in_seconds: u16,
    },
    SetBackgroundColor(SetBackgroundColor),
    SetTabIndex {
        depth: Depth,
        tab_index: u16,
    },
    ShowFrame,
    SoundStreamBlock(SoundStreamBlock<'a>),
    SoundStreamHead(Box<SoundStreamHead>),
    SoundStreamHead2(Box<SoundStreamHead>),
    StartSound(StartSound),
    StartSound2 {
        class_name: &'a SwfStr,
        sound_info: Box<SoundInfo>,
    },
    SymbolClass(Vec<SymbolClassLink<'a>>),
    Unknown {
        tag_code: u16,
        data: &'a [u8],
    },
    VideoFrame(VideoFrame<'a>),
}
