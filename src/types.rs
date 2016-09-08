#[derive(Debug,PartialEq)]
pub struct Swf {
    pub version: u8,
    pub compression: Compression,
    pub stage_size: Rectangle,
    pub frame_rate: f32,
    pub num_frames: u16,
    pub tags: Vec<Tag>,
}

/// Defines the compression type used in an SWF.
#[derive(Debug,PartialEq,Eq)]
pub enum Compression {
    None,
    Zlib,
    Lzma,
}

#[derive(Debug,PartialEq,Clone)]
pub struct Rectangle {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
}

#[derive(Debug,PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug,PartialEq)]
pub struct ColorTransform {
    pub r_multiply: f32,
    pub g_multiply: f32,
    pub b_multiply: f32,
    pub a_multiply: f32,
    pub r_add: i16,
    pub g_add: i16,
    pub b_add: i16,
    pub a_add: i16,
}

#[derive(Debug,PartialEq)]
pub struct Matrix {
    pub translate_x: f32,
    pub translate_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotate_skew_0: f32,
    pub rotate_skew_1: f32,
}

impl Matrix {
    pub fn new() -> Matrix {
        Matrix {
            translate_x: 0f32,
            translate_y: 0f32,
            scale_x: 1f32,
            scale_y: 1f32,
            rotate_skew_0: 0f32,
            rotate_skew_1: 0f32,
        }
    }
}

#[derive(Debug,PartialEq)]
pub struct FileAttributes {
    pub use_direct_blit: bool,
    pub use_gpu: bool,
    pub has_metadata: bool,
    pub is_action_script_3: bool,
    pub use_network_sandbox: bool,
}

#[derive(Debug,PartialEq)]
pub struct FrameLabel {
    pub frame_num: u32,
    pub label: String,
}

pub type Depth = i16;
pub type CharacterId = u16;
pub type Twips = i32;

#[derive(Debug,PartialEq)]
pub struct PlaceObject {
    pub version: u8,
    pub action: PlaceObjectAction,
    pub depth: Depth,
    pub matrix: Option<Matrix>,
    pub color_transform: Option<ColorTransform>,
    pub ratio: Option<u16>,
    pub name: Option<String>,
    pub clip_depth: Option<Depth>,
    pub class_name: Option<String>,
    pub filters: Vec<Filter>,
    pub background_color: Option<Color>,
    pub blend_mode: BlendMode,
    pub clip_actions: Vec<ClipAction>,
    pub is_bitmap_cached: bool,
    pub is_visible: bool,

}

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum PlaceObjectAction {
    Place(CharacterId),
    Modify,
    Replace(CharacterId),
}


#[derive(Debug,PartialEq,Clone)]
pub enum Filter {
    DropShadowFilter,
    BlurFilter,
    GlowFilter,
    BevelFilter,
    GradientGlowFilter,
    ConvolutionFilter,
    ColorMatrixFilter,
    GradientBevelFilter,
}


#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum BlendMode {
    Normal,
    Layer,
    Multiply,
    Screen,
    Lighten,
    Darken,
    Difference,
    Add,
    Subtract,
    Invert,
    Alpha,
    Erase,
    Overlay,
    HardLight,
} 

#[derive(Debug,PartialEq,Eq,Clone)]
pub struct ClipAction {
    pub events: Vec<ClipEvent>,
    pub key_code: Option<u8>,
}

#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum ClipEvent {
    KeyUp,
    KeyDown,
    MouseUp,
    MouseDown,
    MouseMove,
    Unload,
    EnterFrame,
    Load,
    DragOver,
    RollOut,
    RollOver,
    ReleaseOutside,
    Release,
    Press,
    Initialize,
    Construct,
}

#[derive(Debug,PartialEq)]
pub enum Tag {
    ShowFrame,

    DefineShape(Shape),

    SetBackgroundColor(Color),

    PlaceObject(Box<PlaceObject>),

    FileAttributes(FileAttributes),

    DefineSceneAndFrameLabelData {
        scenes: Vec<FrameLabel>,
        frame_labels: Vec<FrameLabel>,
    },

    Unknown { tag_code: u16, data: Vec<u8> },
}

#[derive(Debug,PartialEq)]
pub struct Shape {
    pub version: u8,
    pub id: CharacterId,
    pub shape_bounds: Rectangle,
    pub edge_bounds: Rectangle,
    pub styles: ShapeStyles,
    pub shape: Vec<ShapeRecord>,
}

#[derive(Debug,PartialEq)]
pub struct ShapeStyles {
    pub fill_styles: Vec<FillStyle>,
    pub line_styles: Vec<LineStyle>,
}

#[derive(Debug,PartialEq)]
pub enum ShapeRecord {
    // TODO: Twips
    StyleChange(StyleChangeData),
    StraightEdge { delta_x: f32, delta_y: f32 },
    CurvedEdge {
        control_delta_x: f32,
        control_delta_y: f32,
        anchor_delta_x: f32,
        anchor_delta_y: f32,
    },
}

#[derive(Debug,PartialEq)]
pub struct StyleChangeData {
    pub move_delta_x: f32,
    pub move_delta_y: f32,
    pub fill_style_0: Option<u32>,
    pub fill_style_1: Option<u32>,
    pub line_style: Option<u32>,
    pub new_styles: Option<ShapeStyles>,
}

#[derive(Debug,PartialEq)]
pub enum FillStyle {
    Color(Color),
    LinearGradient(Gradient),
    RadialGradient(Gradient),
    FocalGradient {
        gradient: Gradient,
        focal_point: f32,
    },
    Bitmap {
        id: CharacterId,
        matrix: Matrix,
        is_smoothed: bool,
        is_repeating: bool,
    },
}

#[derive(Debug,PartialEq)]
pub struct Gradient {
    pub spread: GradientSpread,
    pub interpolation: GradientInterpolation,
    pub records: Vec<GradientRecord>,
}

#[derive(Debug,PartialEq)]
pub enum GradientSpread {
    Pad,
    Reflect,
    Repeat,
}

#[derive(Debug,PartialEq)]
pub enum GradientInterpolation {
    RGB,
    LinearRGB,
}

#[derive(Debug,PartialEq)]
pub struct GradientRecord {
    pub ratio: u8,
    pub color: Color,
}

#[derive(Debug,PartialEq)]
pub struct LineStyle {
    pub width: u16, // Twips
    pub color: Color,
}

// TODO: LineStyle2.
#[derive(Debug,PartialEq)]
pub enum LineCapStyle {
    Round,
    None,
    Square,
}
#[derive(Debug,PartialEq)]
pub enum LineJoinStyle {
    Round,
    Bevel,
    Miter(f32),
}