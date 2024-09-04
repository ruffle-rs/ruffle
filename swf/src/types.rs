//! The data structures used in an Adobe SWF file.
//!
//! These structures are documented in the Adobe SWF File Format Specification
//! version 19 (henceforth SWF19):
//! <https://www.adobe.com/content/dam/acom/en/devnet/pdf/swf-file-format-spec.pdf>

use crate::string::SwfStr;
use bitflags::bitflags;
use enum_map::Enum;
use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::num::NonZeroU8;
use std::str::FromStr;

mod bevel_filter;
mod blur_filter;
mod color;
mod color_matrix_filter;
mod color_transform;
mod convolution_filter;
mod drop_shadow_filter;
mod fixed;
mod glow_filter;
mod gradient_filter;
mod matrix;
mod point;
mod rectangle;
mod twips;

pub use bevel_filter::{BevelFilter, BevelFilterFlags};
pub use blur_filter::{BlurFilter, BlurFilterFlags};
pub use color::Color;
pub use color_matrix_filter::ColorMatrixFilter;
pub use color_transform::ColorTransform;
pub use convolution_filter::{ConvolutionFilter, ConvolutionFilterFlags};
pub use drop_shadow_filter::{DropShadowFilter, DropShadowFilterFlags};
pub use fixed::{Fixed16, Fixed8};
pub use glow_filter::{GlowFilter, GlowFilterFlags};
pub use gradient_filter::{GradientFilter, GradientFilterFlags};
pub use matrix::Matrix;
pub use point::{Point, PointDelta};
pub use rectangle::Rectangle;
pub use twips::Twips;

/// A complete header and tags in the SWF file.
/// This is returned by the `swf::parse_swf` convenience method.
#[derive(Debug)]
pub struct Swf<'a> {
    pub header: HeaderExt,
    pub tags: Vec<Tag<'a>>,
}

/// Returned by `read::decompress_swf`.
/// Owns the decompressed SWF data, which will be referenced when parsed by `parse_swf`.
pub struct SwfBuf {
    /// The parsed SWF header.
    pub header: HeaderExt,

    /// The decompressed SWF tag stream.
    pub data: Vec<u8>,
}

/// The header of an SWF file.
///
/// Notably contains the compression format used by the rest of the SWF data.
///
/// [SWF19 p.27](https://www.adobe.com/content/dam/acom/en/devnet/pdf/swf-file-format-spec.pdf#page=27)
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Header {
    pub compression: Compression,
    pub version: u8,
    pub stage_size: Rectangle<Twips>,
    pub frame_rate: Fixed8,
    pub num_frames: u16,
}

impl Header {
    pub fn default_with_swf_version(version: u8) -> Self {
        Self {
            compression: Compression::None,
            version,
            stage_size: Default::default(),
            frame_rate: Fixed8::ONE,
            num_frames: 0,
        }
    }
}

/// The extended metadata of an SWF file.
///
/// This includes the SWF header data as well as metadata from the FileAttributes and
/// SetBackgroundColor tags.
///
/// This metadata may not reflect the actual data inside a malformed SWF; for example,
/// the root timeline my actually contain fewer frames than `HeaderExt::num_frames` if it is
/// corrupted.
#[derive(Clone, Debug)]
pub struct HeaderExt {
    pub(crate) header: Header,
    pub(crate) file_attributes: FileAttributes,
    pub(crate) background_color: Option<SetBackgroundColor>,
    pub(crate) uncompressed_len: i32,
}

impl HeaderExt {
    #[inline]
    /// Returns the header for a dummy SWF file with the given SWF version.
    pub fn default_with_swf_version(version: u8) -> Self {
        Self {
            header: Header::default_with_swf_version(version),
            file_attributes: Default::default(),
            background_color: None,
            uncompressed_len: 0,
        }
    }

    /// Returns the header for the error state movie stub which is used if no file
    /// could be loaded or if the loaded content is no valid supported content.
    pub fn default_error_header() -> Self {
        Self {
            header: Header::default_with_swf_version(0),
            file_attributes: Default::default(),
            background_color: None,
            uncompressed_len: -1,
        }
    }

    /// Returns the header for a loaded image (JPEG, GIF or PNG).
    pub fn default_with_uncompressed_len(length: i32) -> Self {
        let header = Header {
            compression: Compression::None,
            version: 0,
            stage_size: Default::default(),
            frame_rate: Fixed8::ONE,
            num_frames: 1,
        };
        Self {
            header,
            file_attributes: Default::default(),
            background_color: None,
            uncompressed_len: length,
        }
    }

    /// The background color of the SWF from the SetBackgroundColor tag.
    ///
    /// `None` will be returned if the SetBackgroundColor tag was not found.
    #[inline]
    pub fn background_color(&self) -> Option<Color> {
        self.background_color
    }

    /// The compression format used by the SWF.
    #[inline]
    pub fn compression(&self) -> Compression {
        self.header.compression
    }

    /// The frame rate of the SWF, in frames per second.
    #[inline]
    pub fn frame_rate(&self) -> Fixed8 {
        self.header.frame_rate
    }

    /// Whether this SWF contains XMP metadata in a Metadata tag.
    #[inline]
    pub fn has_metadata(&self) -> bool {
        self.file_attributes.contains(FileAttributes::HAS_METADATA)
    }

    /// Returns the basic SWF header.
    #[inline]
    pub fn swf_header(&self) -> &Header {
        &self.header
    }

    /// Whether this SWF uses ActionScript 3.0 (AVM2).
    #[inline]
    pub fn is_action_script_3(&self) -> bool {
        self.file_attributes
            .contains(FileAttributes::IS_ACTION_SCRIPT_3)
    }

    /// The number of frames on the root timeline.
    #[inline]
    pub fn num_frames(&self) -> u16 {
        self.header.num_frames
    }

    /// The stage dimensions of this SWF.
    #[inline]
    pub fn stage_size(&self) -> &Rectangle<Twips> {
        &self.header.stage_size
    }

    /// The SWF version.
    #[inline]
    pub fn version(&self) -> u8 {
        self.header.version
    }

    /// The length of the SWF after decompression.
    #[inline]
    pub fn uncompressed_len(&self) -> i32 {
        self.uncompressed_len
    }

    /// Whether this SWF requests hardware acceleration to blit to the screen.
    #[inline]
    pub fn use_direct_blit(&self) -> bool {
        self.file_attributes
            .contains(FileAttributes::USE_DIRECT_BLIT)
    }

    /// Whether this SWF requests hardware acceleration for compositing.
    #[inline]
    pub fn use_gpu(&self) -> bool {
        self.file_attributes.contains(FileAttributes::USE_GPU)
    }

    /// Whether this SWF should be placed in the network sandbox when run locally.
    ///
    /// SWFs in the network sandbox can only access network resources,  not local resources.
    /// SWFs in the local sandbox can only access local resources, not network resources.
    #[inline]
    pub fn use_network_sandbox(&self) -> bool {
        self.file_attributes
            .contains(FileAttributes::USE_NETWORK_SANDBOX)
    }
}

/// The compression format used internally by the SWF file.
///
/// The vast majority of SWFs will use zlib compression.
/// [SWF19 p.27](https://www.adobe.com/content/dam/acom/en/devnet/pdf/swf-file-format-spec.pdf#page=27)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Compression {
    None,
    Zlib,
    Lzma,
}

#[derive(Clone, Copy, Debug, Eq, FromPrimitive, PartialEq)]
pub enum Language {
    Unknown = 0,
    Latin = 1,
    Japanese = 2,
    Korean = 3,
    SimplifiedChinese = 4,
    TraditionalChinese = 5,
}

impl Language {
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(n)
    }
}

bitflags! {
    /// Flags that define various characteristic of an SWF file.
    ///
    /// [SWF19 pp.57-58 ClipEvent](https://www.adobe.com/content/dam/acom/en/devnet/pdf/swf-file-format-spec.pdf#page=47)
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct FileAttributes: u8 {
        /// Whether this SWF requests hardware acceleration to blit to the screen.
        const USE_DIRECT_BLIT = 1 << 6;

        /// Whether this SWF requests hardware acceleration for compositing.
        const USE_GPU = 1 << 5;

        /// Whether this SWF contains XMP metadata in a Metadata tag.
        const HAS_METADATA = 1 << 4;

        /// Whether this SWF uses ActionScript 3 (AVM2).
        const IS_ACTION_SCRIPT_3 = 1 << 3;

        /// Whether this SWF should be placed in the network sandbox when run locally.
        ///
        /// SWFs in the network sandbox can only access network resources,  not local resources.
        /// SWFs in the local sandbox can only access local resources, not network resources.
        const USE_NETWORK_SANDBOX = 1 << 0;
    }
}

impl Default for FileAttributes {
    fn default() -> Self {
        // The settings for SWF7 and earlier, which contain no FileAttributes tag.
        Self::empty()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct FrameLabel<'a> {
    pub label: &'a SwfStr,
    pub is_anchor: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DefineSceneAndFrameLabelData<'a> {
    pub scenes: Vec<FrameLabelData<'a>>,
    pub frame_labels: Vec<FrameLabelData<'a>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FrameLabelData<'a> {
    pub frame_num: u32,
    pub label: &'a SwfStr,
}

pub type Depth = u16;
pub type CharacterId = u16;

#[derive(Debug, PartialEq)]
pub struct PlaceObject<'a> {
    pub version: u8,
    pub action: PlaceObjectAction,
    pub depth: Depth,
    pub matrix: Option<Matrix>,
    pub color_transform: Option<ColorTransform>,
    pub ratio: Option<u16>,
    pub name: Option<&'a SwfStr>,
    pub clip_depth: Option<Depth>,
    pub class_name: Option<&'a SwfStr>,
    pub filters: Option<Vec<Filter>>,
    pub background_color: Option<Color>,
    pub blend_mode: Option<BlendMode>,
    pub clip_actions: Option<Vec<ClipAction<'a>>>,
    pub has_image: bool,
    pub is_bitmap_cached: Option<bool>,
    pub is_visible: Option<bool>,
    pub amf_data: Option<&'a [u8]>,
}

bitflags! {
    pub struct PlaceFlag: u16 {
        const MOVE = 1 << 0;
        const HAS_CHARACTER = 1 << 1;
        const HAS_MATRIX = 1 << 2;
        const HAS_COLOR_TRANSFORM = 1 << 3;
        const HAS_RATIO = 1 << 4;
        const HAS_NAME = 1 << 5;
        const HAS_CLIP_DEPTH = 1 << 6;
        const HAS_CLIP_ACTIONS = 1 << 7;

        // PlaceObject3
        const HAS_FILTER_LIST = 1 << 8;
        const HAS_BLEND_MODE = 1 << 9;
        const HAS_CACHE_AS_BITMAP = 1 << 10;
        const HAS_CLASS_NAME = 1 << 11;
        const HAS_IMAGE = 1 << 12;
        const HAS_VISIBLE = 1 << 13;
        const OPAQUE_BACKGROUND = 1 << 14;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaceObjectAction {
    Place(CharacterId),
    Modify,
    Replace(CharacterId),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Filter {
    DropShadowFilter(Box<DropShadowFilter>),
    BlurFilter(Box<BlurFilter>),
    GlowFilter(Box<GlowFilter>),
    BevelFilter(Box<BevelFilter>),
    GradientGlowFilter(Box<GradientFilter>),
    ConvolutionFilter(Box<ConvolutionFilter>),
    ColorMatrixFilter(Box<ColorMatrixFilter>),
    GradientBevelFilter(Box<GradientFilter>),
}

#[derive(Default, Clone, Copy, Debug, Eq, FromPrimitive, PartialEq, Enum)]
pub enum BlendMode {
    #[default]
    Normal = 0,
    Layer = 2,
    Multiply = 3,
    Screen = 4,
    Lighten = 5,
    Darken = 6,
    Difference = 7,
    Add = 8,
    Subtract = 9,
    Invert = 10,
    Alpha = 11,
    Erase = 12,
    Overlay = 13,
    HardLight = 14,
}

impl BlendMode {
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(match n {
            1 => 0,
            n => n,
        })
    }
}

impl Display for BlendMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match *self {
            BlendMode::Normal => "normal",
            BlendMode::Layer => "layer",
            BlendMode::Multiply => "multiply",
            BlendMode::Screen => "screen",
            BlendMode::Lighten => "lighten",
            BlendMode::Darken => "darken",
            BlendMode::Difference => "difference",
            BlendMode::Add => "add",
            BlendMode::Subtract => "subtract",
            BlendMode::Invert => "invert",
            BlendMode::Alpha => "alpha",
            BlendMode::Erase => "erase",
            BlendMode::Overlay => "overlay",
            BlendMode::HardLight => "hardlight",
        };
        f.write_str(s)
    }
}

impl FromStr for BlendMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mode = match s {
            "normal" => BlendMode::Normal,
            "layer" => BlendMode::Layer,
            "multiply" => BlendMode::Multiply,
            "screen" => BlendMode::Screen,
            "lighten" => BlendMode::Lighten,
            "darken" => BlendMode::Darken,
            "difference" => BlendMode::Difference,
            "add" => BlendMode::Add,
            "subtract" => BlendMode::Subtract,
            "invert" => BlendMode::Invert,
            "alpha" => BlendMode::Alpha,
            "erase" => BlendMode::Erase,
            "overlay" => BlendMode::Overlay,
            "hardlight" => BlendMode::HardLight,
            _ => return Err(()),
        };
        Ok(mode)
    }
}

/// An clip action (a.k.a. clip event) placed on a MovieClip instance.
/// Created in the Flash IDE using `onClipEvent` or `on` blocks.
///
/// [SWF19 pp.37-38 ClipActionRecord](https://www.adobe.com/content/dam/acom/en/devnet/pdf/swf-file-format-spec.pdf#page=39)
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipAction<'a> {
    pub events: ClipEventFlag,
    pub key_code: Option<KeyCode>,
    pub action_data: &'a [u8],
}

bitflags! {
    /// An event that can be attached to a MovieClip instance using an `onClipEvent` or `on` block.
    ///
    /// [SWF19 pp.48-50 ClipEvent](https://www.adobe.com/content/dam/acom/en/devnet/pdf/swf-file-format-spec.pdf#page=50)
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct ClipEventFlag: u32 {
        const LOAD            = 1 << 0;
        const ENTER_FRAME     = 1 << 1;
        const UNLOAD          = 1 << 2;
        const MOUSE_MOVE      = 1 << 3;
        const MOUSE_DOWN      = 1 << 4;
        const MOUSE_UP        = 1 << 5;
        const KEY_DOWN        = 1 << 6;
        const KEY_UP          = 1 << 7;

        // Added in SWF6, but not version-gated.
        const DATA            = 1 << 8;
        const INITIALIZE      = 1 << 9;
        const PRESS           = 1 << 10;
        const RELEASE         = 1 << 11;
        const RELEASE_OUTSIDE = 1 << 12;
        const ROLL_OVER       = 1 << 13;
        const ROLL_OUT        = 1 << 14;
        const DRAG_OVER       = 1 << 15;
        const DRAG_OUT        = 1 << 16;
        const KEY_PRESS       = 1 << 17;

        // Construct was only added in SWF7, but it's not version-gated;
        // Construct events will still fire in SWF6 in a v7+ player (#1424).
        const CONSTRUCT       = 1 << 18;
    }
}

/// A key code used in `ButtonAction` and `ClipAction` key press events.
pub type KeyCode = u8;

/// Represents a tag in an SWF file.
///
/// The SWF format is made up of a stream of tags. Each tag either
/// defines a character (Graphic, Sound, MovieClip), or places/modifies
/// an instance of these characters on the display list.
///
// [SWF19 p.29](https://www.adobe.com/content/dam/acom/en/devnet/pdf/swf-file-format-spec.pdf#page=29)
#[derive(Debug, PartialEq)]
pub enum Tag<'a> {
    ExportAssets(ExportAssets<'a>),
    ScriptLimits {
        max_recursion_depth: u16,
        timeout_in_seconds: u16,
    },
    ShowFrame,

    Protect(Option<&'a SwfStr>),
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
    DefineShape(Shape),
    DefineSound(Box<Sound<'a>>),
    DefineSprite(Sprite<'a>),
    DefineText(Box<Text>),
    DefineText2(Box<Text>),
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
    Metadata(&'a SwfStr),
    ImportAssets {
        url: &'a SwfStr,
        imports: Vec<ExportedAsset<'a>>,
    },
    JpegTables(JpegTables<'a>),
    NameCharacter(NameCharacter<'a>),
    SetBackgroundColor(SetBackgroundColor),
    SetTabIndex {
        depth: Depth,
        tab_index: u16,
    },
    SoundStreamBlock(SoundStreamBlock<'a>),
    SoundStreamHead(Box<SoundStreamHead>),
    SoundStreamHead2(Box<SoundStreamHead>),
    StartSound(StartSound),
    StartSound2 {
        class_name: &'a SwfStr,
        sound_info: Box<SoundInfo>,
    },
    SymbolClass(Vec<SymbolClassLink<'a>>),
    PlaceObject(Box<PlaceObject<'a>>),
    RemoveObject(RemoveObject),
    VideoFrame(VideoFrame<'a>),
    FileAttributes(FileAttributes),

    FrameLabel(FrameLabel<'a>),
    DefineSceneAndFrameLabelData(DefineSceneAndFrameLabelData<'a>),

    ProductInfo(ProductInfo),

    Unknown {
        tag_code: u16,
        data: &'a [u8],
    },
}

pub type ExportAssets<'a> = Vec<ExportedAsset<'a>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportedAsset<'a> {
    pub id: CharacterId,
    pub name: &'a SwfStr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoveObject {
    pub depth: Depth,
    pub character_id: Option<CharacterId>,
}

pub type SetBackgroundColor = Color;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SymbolClassLink<'a> {
    pub id: CharacterId,
    pub class_name: &'a SwfStr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShapeContext {
    pub swf_version: u8,
    pub shape_version: u8,
    pub num_fill_bits: u8,
    pub num_line_bits: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Shape {
    pub version: u8,
    pub id: CharacterId,
    pub shape_bounds: Rectangle<Twips>,
    pub edge_bounds: Rectangle<Twips>,
    pub flags: ShapeFlag,
    pub styles: ShapeStyles,
    pub shape: Vec<ShapeRecord>,
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct ShapeFlag: u8 {
        const HAS_SCALING_STROKES     = 1 << 0;
        const HAS_NON_SCALING_STROKES = 1 << 1;
        const NON_ZERO_WINDING_RULE   = 1 << 2;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sound<'a> {
    pub id: CharacterId,
    pub format: SoundFormat,
    pub num_samples: u32,
    pub data: &'a [u8],
}

#[derive(Clone, Debug, PartialEq)]
pub struct SoundInfo {
    pub event: SoundEvent,
    pub in_sample: Option<u32>,
    pub out_sample: Option<u32>,
    pub num_loops: u16,
    pub envelope: Option<SoundEnvelope>,
}

#[derive(Clone, Copy, Debug, Eq, FromPrimitive, PartialEq)]
pub enum SoundEvent {
    Event = 0,
    Start = 1,
    Stop = 2,
}

impl SoundEvent {
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(match n {
            3 => 2,
            n => n,
        })
    }
}

pub type SoundEnvelope = Vec<SoundEnvelopePoint>;

#[derive(Clone, Debug, PartialEq)]
pub struct SoundEnvelopePoint {
    pub sample: u32,
    pub left_volume: f32,
    pub right_volume: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StartSound {
    pub id: CharacterId,
    pub sound_info: Box<SoundInfo>,
}

#[derive(Debug, PartialEq)]
pub struct Sprite<'a> {
    pub id: CharacterId,
    pub num_frames: u16,
    pub tags: Vec<Tag<'a>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShapeStyles {
    pub fill_styles: Vec<FillStyle>,
    pub line_styles: Vec<LineStyle>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShapeRecord {
    StyleChange(Box<StyleChangeData>),
    StraightEdge {
        delta: PointDelta<Twips>,
    },
    CurvedEdge {
        control_delta: PointDelta<Twips>,
        anchor_delta: PointDelta<Twips>,
    },
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct ShapeRecordFlag: u8 {
        const MOVE_TO      = 1 << 0;
        const FILL_STYLE_0 = 1 << 1;
        const FILL_STYLE_1 = 1 << 2;
        const LINE_STYLE   = 1 << 3;
        const NEW_STYLES   = 1 << 4;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StyleChangeData {
    pub move_to: Option<Point<Twips>>,
    pub fill_style_0: Option<u32>,
    pub fill_style_1: Option<u32>,
    pub line_style: Option<u32>,
    pub new_styles: Option<ShapeStyles>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FillStyle {
    Color(Color),
    LinearGradient(Gradient),
    RadialGradient(Gradient),
    FocalGradient {
        gradient: Gradient,
        focal_point: Fixed8,
    },
    Bitmap {
        id: CharacterId,
        matrix: Matrix,
        is_smoothed: bool,
        is_repeating: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Gradient {
    pub matrix: Matrix,
    pub spread: GradientSpread,
    pub interpolation: GradientInterpolation,
    pub records: Vec<GradientRecord>,
}

#[derive(Clone, Copy, Debug, Eq, FromPrimitive, PartialEq, Enum, Hash)]
pub enum GradientSpread {
    Pad = 0,
    Reflect = 1,
    Repeat = 2,
}

impl GradientSpread {
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(match n {
            // Per SWF19 p. 136, SpreadMode 3 is reserved.
            // Flash treats it as pad mode.
            3 => 0,
            n => n,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, FromPrimitive, PartialEq, Hash)]
pub enum GradientInterpolation {
    Rgb = 0,
    LinearRgb = 1,
}

impl GradientInterpolation {
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(match n {
            // Per SWF19 p. 136, InterpolationMode 2 and 3 are reserved.
            // Flash treats them as normal RGB mode interpolation.
            2 | 3 => 0,
            n => n,
        })
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct GradientRecord {
    pub ratio: u8,
    pub color: Color,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineStyle {
    pub(crate) width: Twips,
    pub(crate) fill_style: FillStyle,
    pub(crate) flags: LineStyleFlag,
    pub(crate) miter_limit: Fixed8,
}

impl LineStyle {
    #[inline]
    pub fn new() -> LineStyle {
        Default::default()
    }

    #[inline]
    pub fn allow_close(&self) -> bool {
        !self.flags.contains(LineStyleFlag::NO_CLOSE)
    }

    #[inline]
    pub fn with_allow_close(mut self, val: bool) -> Self {
        self.flags.set(LineStyleFlag::NO_CLOSE, !val);
        self
    }

    #[inline]
    pub fn allow_scale_x(&self) -> bool {
        !self.flags.contains(LineStyleFlag::NO_H_SCALE)
    }

    #[inline]
    pub fn with_allow_scale_x(mut self, val: bool) -> Self {
        self.flags.set(LineStyleFlag::NO_H_SCALE, !val);
        self
    }

    #[inline]
    pub fn allow_scale_y(&self) -> bool {
        !self.flags.contains(LineStyleFlag::NO_V_SCALE)
    }

    #[inline]
    pub fn with_allow_scale_y(mut self, val: bool) -> Self {
        self.flags.set(LineStyleFlag::NO_V_SCALE, !val);
        self
    }

    #[inline]
    pub fn is_pixel_hinted(&self) -> bool {
        self.flags.contains(LineStyleFlag::PIXEL_HINTING)
    }

    #[inline]
    pub fn with_is_pixel_hinted(mut self, val: bool) -> Self {
        self.flags.set(LineStyleFlag::PIXEL_HINTING, val);
        self
    }

    #[inline]
    pub fn start_cap(&self) -> LineCapStyle {
        let cap = (self.flags & LineStyleFlag::START_CAP_STYLE).bits() >> 6;
        LineCapStyle::from_u8(cap as u8).unwrap()
    }

    #[inline]
    pub fn with_start_cap(mut self, val: LineCapStyle) -> Self {
        self.flags -= LineStyleFlag::START_CAP_STYLE;
        self.flags |= LineStyleFlag::from_bits_retain((val as u16) << 6);
        self
    }

    #[inline]
    pub fn end_cap(&self) -> LineCapStyle {
        let cap = (self.flags & LineStyleFlag::END_CAP_STYLE).bits() >> 8;
        LineCapStyle::from_u8(cap as u8).unwrap()
    }

    #[inline]
    pub fn with_end_cap(mut self, val: LineCapStyle) -> Self {
        self.flags -= LineStyleFlag::END_CAP_STYLE;
        self.flags |= LineStyleFlag::from_bits_retain((val as u16) << 8);
        self
    }

    #[inline]
    pub fn join_style(&self) -> LineJoinStyle {
        match self.flags & LineStyleFlag::JOIN_STYLE {
            LineStyleFlag::ROUND => LineJoinStyle::Round,
            LineStyleFlag::BEVEL => LineJoinStyle::Bevel,
            LineStyleFlag::MITER => LineJoinStyle::Miter(self.miter_limit),
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn with_join_style(mut self, val: LineJoinStyle) -> Self {
        self.flags -= LineStyleFlag::JOIN_STYLE;
        self.flags |= match val {
            LineJoinStyle::Round => LineStyleFlag::ROUND,
            LineJoinStyle::Bevel => LineStyleFlag::BEVEL,
            LineJoinStyle::Miter(miter_limit) => {
                self.miter_limit = miter_limit;
                LineStyleFlag::MITER
            }
        };
        self
    }

    #[inline]
    pub fn fill_style(&self) -> &FillStyle {
        &self.fill_style
    }

    #[inline]
    pub fn with_fill_style(mut self, val: FillStyle) -> Self {
        self.flags
            .set(LineStyleFlag::HAS_FILL, !matches!(val, FillStyle::Color(_)));
        self.fill_style = val;
        self
    }

    #[inline]
    pub fn with_color(mut self, val: Color) -> Self {
        self.flags.remove(LineStyleFlag::HAS_FILL);
        self.fill_style = FillStyle::Color(val);
        self
    }

    #[inline]
    pub fn width(&self) -> Twips {
        self.width
    }

    #[inline]
    pub fn with_width(mut self, val: Twips) -> Self {
        self.width = val;
        self
    }
}

impl Default for LineStyle {
    #[inline]
    fn default() -> Self {
        // Hairline black stroke.
        Self {
            width: Twips::ZERO,
            fill_style: FillStyle::Color(Color::BLACK),
            flags: Default::default(),
            miter_limit: Default::default(),
        }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct LineStyleFlag: u16 {
        // First byte.
        const PIXEL_HINTING = 1 << 0;
        const NO_V_SCALE = 1 << 1;
        const NO_H_SCALE = 1 << 2;
        const HAS_FILL = 1 << 3;
        const JOIN_STYLE = 0b11 << 4;
        const START_CAP_STYLE = 0b11 << 6;

        // Second byte.
        const END_CAP_STYLE = 0b11 << 8;
        const NO_CLOSE = 1 << 10;

        // JOIN_STYLE mask values.
        const ROUND = 0b00 << 4;
        const BEVEL = 0b01 << 4;
        const MITER = 0b10 << 4;
    }
}

impl Default for LineStyleFlag {
    #[inline]
    fn default() -> Self {
        LineStyleFlag::empty()
    }
}

#[derive(Default, Clone, Copy, Debug, Eq, FromPrimitive, PartialEq)]
pub enum LineCapStyle {
    #[default]
    Round = 0,
    None = 1,
    Square = 2,
}

impl LineCapStyle {
    #[inline]
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(n)
    }
}

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineJoinStyle {
    #[default]
    Round,
    Bevel,
    Miter(Fixed8),
}

#[derive(Clone, Copy, Debug, Eq, FromPrimitive, PartialEq)]
pub enum AudioCompression {
    UncompressedUnknownEndian = 0,
    Adpcm = 1,
    Mp3 = 2,
    Uncompressed = 3,
    Nellymoser16Khz = 4,
    Nellymoser8Khz = 5,
    Nellymoser = 6,
    Speex = 11,
}

impl AudioCompression {
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(n)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SoundFormat {
    pub compression: AudioCompression,
    pub sample_rate: u16,
    pub is_stereo: bool,
    pub is_16_bit: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SoundStreamHead {
    pub stream_format: SoundFormat,
    pub playback_format: SoundFormat,
    pub num_samples_per_block: u16,
    pub latency_seek: i16,
}

pub type SoundStreamBlock<'a> = &'a [u8];

#[derive(Clone, Debug, PartialEq)]
pub struct Button<'a> {
    pub id: CharacterId,
    pub is_track_as_menu: bool,
    pub records: Vec<ButtonRecord>,
    pub actions: Vec<ButtonAction<'a>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonRecord {
    pub states: ButtonState,
    pub id: CharacterId,
    pub depth: Depth,
    pub matrix: Matrix,
    pub color_transform: ColorTransform,
    pub filters: Vec<Filter>,
    pub blend_mode: BlendMode,
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct ButtonState: u8 {
        const UP       = 1 << 0;
        const OVER     = 1 << 1;
        const DOWN     = 1 << 2;
        const HIT_TEST = 1 << 3;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ButtonColorTransform {
    pub id: CharacterId,
    pub color_transforms: Vec<ColorTransform>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonSounds {
    pub id: CharacterId,
    pub over_to_up_sound: Option<ButtonSound>,
    pub up_to_over_sound: Option<ButtonSound>,
    pub over_to_down_sound: Option<ButtonSound>,
    pub down_to_over_sound: Option<ButtonSound>,
}

pub type ButtonSound = (CharacterId, SoundInfo);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ButtonAction<'a> {
    pub conditions: ButtonActionCondition,
    pub action_data: &'a [u8],
}

impl ButtonAction<'_> {
    pub fn key_press(&self) -> Option<NonZeroU8> {
        NonZeroU8::new(((self.conditions & ButtonActionCondition::KEY_PRESS).bits() >> 9) as u8)
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct ButtonActionCondition: u16 {
        const IDLE_TO_OVER_UP       = 1 << 0;
        const OVER_UP_TO_IDLE       = 1 << 1;
        const OVER_UP_TO_OVER_DOWN  = 1 << 2;
        const OVER_DOWN_TO_OVER_UP  = 1 << 3;
        const OVER_DOWN_TO_OUT_DOWN = 1 << 4;
        const OUT_DOWN_TO_OVER_DOWN = 1 << 5;
        const OUT_DOWN_TO_IDLE      = 1 << 6;
        const IDLE_TO_OVER_DOWN     = 1 << 7;
        const OVER_DOWN_TO_IDLE     = 1 << 8;
        const KEY_PRESS             = 0b1111111 << 9;
    }
}

impl ButtonActionCondition {
    #[inline]
    pub fn from_key_code(key_code: u8) -> Self {
        Self::from_bits_retain((key_code as u16) << 9)
    }

    /// Checks if the given test condition matches any of the conditions defined in self
    #[inline]
    pub fn matches(self, test_condition: ButtonActionCondition) -> bool {
        let self_key_press = (self & Self::KEY_PRESS).bits();
        let test_key_press = (test_condition & Self::KEY_PRESS).bits();
        let self_without_key = self & !Self::KEY_PRESS;
        let test_without_key = test_condition & !Self::KEY_PRESS;

        self_without_key.contains(test_without_key)
            && (test_key_press == 0 || test_key_press == self_key_press)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DefineMorphShape {
    pub version: u8,
    pub id: CharacterId,
    pub flags: DefineMorphShapeFlag,
    pub start: MorphShape,
    pub end: MorphShape,
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct DefineMorphShapeFlag: u8 {
        const HAS_SCALING_STROKES     = 1 << 0;
        const HAS_NON_SCALING_STROKES = 1 << 1;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MorphShape {
    pub shape_bounds: Rectangle<Twips>,
    pub edge_bounds: Rectangle<Twips>,
    pub fill_styles: Vec<FillStyle>,
    pub line_styles: Vec<LineStyle>,
    pub shape: Vec<ShapeRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontV1 {
    pub id: CharacterId,
    pub glyphs: Vec<Vec<ShapeRecord>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Font<'a> {
    pub version: u8,
    pub id: CharacterId,
    pub name: &'a SwfStr,
    pub language: Language,
    pub layout: Option<FontLayout>,
    pub glyphs: Vec<Glyph>,
    pub flags: FontFlag,
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct FontFlag: u8 {
        const IS_BOLD = 1 << 0;
        const IS_ITALIC = 1 << 1;
        const HAS_WIDE_CODES = 1 << 2;
        const HAS_WIDE_OFFSETS = 1 << 3;
        const IS_ANSI = 1 << 4;
        const IS_SMALL_TEXT = 1 << 5;
        const IS_SHIFT_JIS = 1 << 6;
        const HAS_LAYOUT = 1 << 7;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Font4<'a> {
    pub id: CharacterId,
    pub is_italic: bool,
    pub is_bold: bool,
    pub name: &'a SwfStr,
    pub data: Option<&'a [u8]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Glyph {
    pub shape_records: Vec<ShapeRecord>,
    pub code: u16,
    pub advance: i16,
    pub bounds: Option<Rectangle<Twips>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontLayout {
    pub ascent: u16,
    pub descent: u16,
    pub leading: i16,
    pub kerning: Vec<KerningRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KerningRecord {
    pub left_code: u16,
    pub right_code: u16,
    pub adjustment: Twips,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontInfo<'a> {
    pub id: CharacterId,
    pub version: u8,
    pub name: &'a SwfStr,
    pub flags: FontInfoFlag,
    pub language: Language,
    pub code_table: Vec<u16>,
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct FontInfoFlag: u8 {
        const HAS_WIDE_CODES = 1 << 0;
        const IS_BOLD = 1 << 1;
        const IS_ITALIC = 1 << 2;
        const IS_SHIFT_JIS = 1 << 3;
        const IS_ANSI = 1 << 4;
        const IS_SMALL_TEXT = 1 << 5;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DefineBinaryData<'a> {
    pub id: CharacterId,
    pub data: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Text {
    pub id: CharacterId,
    pub bounds: Rectangle<Twips>,
    pub matrix: Matrix,
    pub records: Vec<TextRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextRecord {
    pub font_id: Option<CharacterId>,
    pub color: Option<Color>,
    pub x_offset: Option<Twips>,
    pub y_offset: Option<Twips>,
    pub height: Option<Twips>,
    pub glyphs: Vec<GlyphEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlyphEntry {
    pub index: u32,
    pub advance: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditText<'a> {
    pub(crate) id: CharacterId,
    pub(crate) bounds: Rectangle<Twips>,
    pub(crate) font_id: CharacterId,
    pub(crate) font_class: &'a SwfStr,
    pub(crate) height: Twips,
    pub(crate) color: Color,
    pub(crate) max_length: u16,
    pub(crate) layout: TextLayout,
    pub(crate) variable_name: &'a SwfStr,
    pub(crate) initial_text: &'a SwfStr,
    pub(crate) flags: EditTextFlag,
}

impl<'a> EditText<'a> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn bounds(&self) -> &Rectangle<Twips> {
        &self.bounds
    }

    #[inline]
    pub fn with_bounds(mut self, bounds: Rectangle<Twips>) -> Self {
        self.bounds = bounds;
        self
    }

    #[inline]
    pub fn id(&self) -> CharacterId {
        self.id
    }

    #[inline]
    pub fn with_id(mut self, id: CharacterId) -> Self {
        self.id = id;
        self
    }

    #[inline]
    pub fn font_id(&self) -> Option<CharacterId> {
        self.flags
            .contains(EditTextFlag::HAS_FONT)
            .then_some(self.font_id)
    }

    #[inline]
    pub fn font_class(&self) -> Option<&'a SwfStr> {
        self.flags
            .contains(EditTextFlag::HAS_FONT_CLASS)
            .then_some(self.font_class)
    }

    // The height of the font in twips.
    #[inline]
    pub fn height(&self) -> Option<Twips> {
        self.flags
            .intersects(EditTextFlag::HAS_FONT | EditTextFlag::HAS_FONT_CLASS)
            .then_some(self.height)
    }

    #[inline]
    pub fn with_default_font(mut self) -> Self {
        self.flags -= EditTextFlag::HAS_FONT | EditTextFlag::HAS_FONT_CLASS;
        self
    }

    #[inline]
    pub fn with_font_id(mut self, font_id: CharacterId, height: Twips) -> Self {
        self.flags |= EditTextFlag::HAS_FONT;
        self.flags -= EditTextFlag::HAS_FONT_CLASS;
        self.font_id = font_id;
        self.height = height;
        self
    }

    #[inline]
    pub fn with_font_class(mut self, font_class: &'a SwfStr, height: Twips) -> Self {
        self.flags |= EditTextFlag::HAS_FONT_CLASS;
        self.flags -= EditTextFlag::HAS_FONT;
        self.font_class = font_class;
        self.height = height;
        self
    }

    #[inline]
    pub fn color(&self) -> Option<&Color> {
        self.flags
            .contains(EditTextFlag::HAS_TEXT_COLOR)
            .then_some(&self.color)
    }

    #[inline]
    pub fn with_color(mut self, color: Option<Color>) -> Self {
        if let Some(color) = color {
            self.flags |= EditTextFlag::HAS_TEXT_COLOR;
            self.color = color;
        } else {
            self.flags -= EditTextFlag::HAS_TEXT_COLOR;
        }
        self
    }

    #[inline]
    pub fn max_length(&self) -> Option<u16> {
        self.flags
            .contains(EditTextFlag::HAS_MAX_LENGTH)
            .then_some(self.max_length)
    }

    #[inline]
    pub fn with_max_length(mut self, max_length: Option<u16>) -> Self {
        if let Some(max_length) = max_length {
            self.flags |= EditTextFlag::HAS_MAX_LENGTH;
            self.max_length = max_length;
        } else {
            self.flags -= EditTextFlag::HAS_MAX_LENGTH;
        }
        self
    }

    #[inline]
    pub fn layout(&self) -> Option<&TextLayout> {
        self.flags
            .contains(EditTextFlag::HAS_LAYOUT)
            .then_some(&self.layout)
    }

    #[inline]
    pub fn with_layout(mut self, layout: Option<TextLayout>) -> Self {
        if let Some(layout) = layout {
            self.flags |= EditTextFlag::HAS_LAYOUT;
            self.layout = layout;
        } else {
            self.flags -= EditTextFlag::HAS_LAYOUT;
        }
        self
    }

    #[inline]
    pub fn variable_name(&self) -> &'a SwfStr {
        self.variable_name
    }

    #[inline]
    pub fn with_variable_name(mut self, variable_name: &'a SwfStr) -> Self {
        self.variable_name = variable_name;
        self
    }

    #[inline]
    pub fn initial_text(&self) -> Option<&'a SwfStr> {
        self.flags
            .contains(EditTextFlag::HAS_TEXT)
            .then_some(self.initial_text)
    }

    #[inline]
    pub fn with_initial_text(mut self, initial_text: Option<&'a SwfStr>) -> Self {
        if let Some(initial_text) = initial_text {
            self.flags |= EditTextFlag::HAS_TEXT;
            self.initial_text = initial_text;
        } else {
            self.flags -= EditTextFlag::HAS_TEXT;
        }
        self
    }

    #[inline]
    pub fn flags(&self) -> EditTextFlag {
        self.flags
    }

    #[inline]
    pub fn is_auto_size(&self) -> bool {
        self.flags.contains(EditTextFlag::AUTO_SIZE)
    }

    #[inline]
    pub fn with_is_auto_size(mut self, value: bool) -> Self {
        self.flags.set(EditTextFlag::AUTO_SIZE, value);
        self
    }

    #[inline]
    pub fn use_outlines(&self) -> bool {
        self.flags.contains(EditTextFlag::USE_OUTLINES)
    }

    #[inline]
    pub fn with_use_outlines(mut self, value: bool) -> Self {
        self.flags.set(EditTextFlag::USE_OUTLINES, value);
        self
    }

    #[inline]
    pub fn has_border(&self) -> bool {
        self.flags.contains(EditTextFlag::BORDER)
    }

    #[inline]
    pub fn with_has_border(mut self, value: bool) -> Self {
        self.flags.set(EditTextFlag::BORDER, value);
        self
    }

    #[inline]
    pub fn is_html(&self) -> bool {
        self.flags.contains(EditTextFlag::HTML)
    }

    #[inline]
    pub fn with_is_html(mut self, value: bool) -> Self {
        self.flags.set(EditTextFlag::HTML, value);
        self
    }

    #[inline]
    pub fn is_multiline(&self) -> bool {
        self.flags.contains(EditTextFlag::MULTILINE)
    }

    #[inline]
    pub fn with_is_multiline(mut self, value: bool) -> Self {
        self.flags.set(EditTextFlag::MULTILINE, value);
        self
    }

    #[inline]
    pub fn is_password(&self) -> bool {
        self.flags.contains(EditTextFlag::PASSWORD)
    }

    #[inline]
    pub fn with_is_password(mut self, value: bool) -> Self {
        self.flags.set(EditTextFlag::PASSWORD, value);
        self
    }

    #[inline]
    pub fn is_read_only(&self) -> bool {
        self.flags.contains(EditTextFlag::READ_ONLY)
    }

    #[inline]
    pub fn with_is_read_only(mut self, value: bool) -> Self {
        self.flags.set(EditTextFlag::READ_ONLY, value);
        self
    }

    #[inline]
    pub fn is_selectable(&self) -> bool {
        !self.flags.contains(EditTextFlag::NO_SELECT)
    }

    #[inline]
    pub fn with_is_selectable(mut self, value: bool) -> Self {
        self.flags.set(EditTextFlag::NO_SELECT, !value);
        self
    }

    #[inline]
    pub fn was_static(&self) -> bool {
        self.flags.contains(EditTextFlag::WAS_STATIC)
    }

    #[inline]
    pub fn with_was_static(mut self, value: bool) -> Self {
        self.flags.set(EditTextFlag::WAS_STATIC, value);
        self
    }

    #[inline]
    pub fn is_word_wrap(&self) -> bool {
        self.flags.contains(EditTextFlag::WORD_WRAP)
    }

    #[inline]
    pub fn with_is_word_wrap(mut self, value: bool) -> Self {
        self.flags.set(EditTextFlag::WORD_WRAP, value);
        self
    }
}

impl<'a> Default for EditText<'a> {
    fn default() -> Self {
        Self {
            id: Default::default(),
            bounds: Default::default(),
            font_id: Default::default(),
            font_class: Default::default(),
            height: Twips::ZERO,
            color: Color::BLACK,
            max_length: 0,
            layout: TextLayout {
                align: TextAlign::Left,
                left_margin: Twips::ZERO,
                right_margin: Twips::ZERO,
                indent: Twips::ZERO,
                leading: Twips::ZERO,
            },
            variable_name: Default::default(),
            initial_text: Default::default(),
            flags: EditTextFlag::empty(),
        }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct EditTextFlag: u16 {
        const HAS_FONT = 1 << 0;
        const HAS_MAX_LENGTH = 1 << 1;
        const HAS_TEXT_COLOR = 1 << 2;
        const READ_ONLY = 1 << 3;
        const PASSWORD = 1 << 4;
        const MULTILINE = 1 << 5;
        const WORD_WRAP = 1 << 6;
        const HAS_TEXT = 1 << 7;

        const USE_OUTLINES = 1 << 8;
        const HTML = 1 << 9;
        const WAS_STATIC = 1 << 10;
        const BORDER = 1 << 11;
        const NO_SELECT = 1 << 12;
        const HAS_LAYOUT = 1 << 13;
        const AUTO_SIZE = 1 << 14;
        const HAS_FONT_CLASS = 1 << 15;
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TextLayout {
    pub align: TextAlign,
    pub left_margin: Twips,
    pub right_margin: Twips,
    pub indent: Twips,
    pub leading: Twips,
}

#[derive(Clone, Copy, Debug, Default, Eq, FromPrimitive, PartialEq)]
pub enum TextAlign {
    #[default]
    Left = 0,
    Right = 1,
    Center = 2,
    Justify = 3,
}

impl TextAlign {
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(n)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontAlignZone {
    // TODO(Herschel): Read these as f16s.
    pub left: i16,
    pub width: i16,
    pub bottom: i16,
    pub height: i16,
}

#[derive(Clone, Copy, Debug, Eq, FromPrimitive, PartialEq)]
pub enum FontThickness {
    Thin = 0,
    Medium = 1,
    Thick = 2,
}

impl FontThickness {
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(n)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CsmTextSettings {
    pub id: CharacterId,
    pub use_advanced_rendering: bool,
    pub grid_fit: TextGridFit,
    pub thickness: f32, // TODO(Herschel): 0.0 is default. Should be Option?
    pub sharpness: f32,
}

#[derive(Clone, Copy, Debug, Eq, FromPrimitive, PartialEq)]
pub enum TextGridFit {
    None = 0,
    Pixel = 1,
    SubPixel = 2,
}

impl TextGridFit {
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(n)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DefineBitsLossless<'a> {
    pub version: u8,
    pub id: CharacterId,
    pub format: BitmapFormat,
    pub width: u16,
    pub height: u16,
    pub data: Cow<'a, [u8]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BitmapFormat {
    ColorMap8 { num_colors: u8 },
    Rgb15,
    Rgb32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DefineVideoStream {
    pub id: CharacterId,
    pub num_frames: u16,
    pub width: u16,
    pub height: u16,
    pub is_smoothed: bool,
    pub deblocking: VideoDeblocking,
    pub codec: VideoCodec,
}

#[derive(Clone, Copy, Debug, Eq, FromPrimitive, PartialEq)]
pub enum VideoDeblocking {
    UseVideoPacketValue = 0,
    None = 1,
    Level1 = 2,
    Level2 = 3,
    Level3 = 4,
    Level4 = 5,
}

impl VideoDeblocking {
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(n)
    }
}

#[derive(Clone, Copy, Debug, Eq, FromPrimitive, PartialEq)]
pub enum VideoCodec {
    None = 0,
    H263 = 2,
    ScreenVideo = 3,
    Vp6 = 4,
    Vp6WithAlpha = 5,
    ScreenVideoV2 = 6,
    H264 = 7,
}

impl VideoCodec {
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(n)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoFrame<'a> {
    pub stream_id: CharacterId,
    pub frame_num: u16,
    pub data: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DefineBitsJpeg3<'a> {
    pub id: CharacterId,
    pub version: u8,
    pub deblocking: Fixed8,
    pub data: &'a [u8],
    pub alpha_data: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoAbc2<'a> {
    pub flags: DoAbc2Flag,
    pub name: &'a SwfStr,
    pub data: &'a [u8],
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct DoAbc2Flag: u32 {
        const LAZY_INITIALIZE = 1 << 0;
    }
}

pub type DoAction<'a> = &'a [u8];

pub type JpegTables<'a> = &'a [u8];

/// Contains information about the software used to generate the SWF.
///
/// Not documented in the SWF19 reference. Emitted by mxmlc.
/// See <http://wahlers.com.br/claus/blog/undocumented-swf-tags-written-by-mxmlc/>
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductInfo {
    pub product_id: u32,
    pub edition: u32,
    pub major_version: u8,
    pub minor_version: u8,
    pub build_number: u64,
    pub compilation_date: u64,
}

/// `DebugId` is a UUID written to debug SWFs and used by the Flash Debugger.
pub type DebugId = [u8; 16];

/// An undocumented and unused tag to set the instance name of a character.
/// This seems to have no effect in the official Flash Player.
/// Superseded by the PlaceObject2 tag.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NameCharacter<'a> {
    pub id: CharacterId,
    pub name: &'a SwfStr,
}

#[cfg(test)]
mod tests {
    use super::ButtonActionCondition;

    #[test]
    fn button_conditions_match() {
        assert!(ButtonActionCondition::OVER_DOWN_TO_OVER_UP
            .matches(ButtonActionCondition::OVER_DOWN_TO_OVER_UP));

        assert!(!ButtonActionCondition::OVER_DOWN_TO_OVER_UP
            .matches(ButtonActionCondition::IDLE_TO_OVER_UP));

        assert!((ButtonActionCondition::OVER_DOWN_TO_OVER_UP
            | ButtonActionCondition::IDLE_TO_OVER_UP)
            .matches(ButtonActionCondition::IDLE_TO_OVER_UP));

        assert!((ButtonActionCondition::OVER_DOWN_TO_OVER_UP
            | ButtonActionCondition::from_key_code(3))
        .matches(ButtonActionCondition::OVER_DOWN_TO_OVER_UP));

        assert!((ButtonActionCondition::OVER_DOWN_TO_OVER_UP
            | ButtonActionCondition::from_key_code(3))
        .matches(ButtonActionCondition::from_key_code(3)));

        assert!(!(ButtonActionCondition::OVER_DOWN_TO_OVER_UP
            | ButtonActionCondition::from_key_code(3))
        .matches(ButtonActionCondition::from_key_code(1)));
    }
}
