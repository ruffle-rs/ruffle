//! Parsed H.263 bitstream types

use std::cmp::Ordering;
use std::ops::{Add, Div, Neg};

/// ITU-T Recommendation H.263 (01/2005) 5.1.2-5.1.4 `TR`, `PTYPE`, `PLUSPTYPE`
/// and 5.1.8 `ETR`.
///
/// The `Picture` configures the current displayed frame's various options,
/// such as it's resolution, the use of any optional H.263 features, and the
/// intra-prediction mode used.
#[derive(Debug)]
pub struct Picture {
    /// The version code.
    ///
    /// Only Sorenson Spark bitstreams contain a version code; compliant H.263
    /// bitstreams are unversioned.
    pub version: Option<u8>,

    /// The temporal reference index of this picture.
    ///
    /// This value may either be 8 or 10 bits wide. This means that references
    /// will overflow after frame 255 or 1023.
    pub temporal_reference: u16,

    /// The source format of the image. Determines it's resolution and frame
    /// rate.
    ///
    /// If unspecified, then the source format matches the reference picture
    /// for this picture.
    pub format: Option<SourceFormat>,

    /// Options which are enabled (or were implicitly present) on this picture.
    pub options: PictureOption,

    /// Indicates if this picture was sent with a `PLUSPTYPE`.
    pub has_plusptype: bool,

    /// Indicates if this picture was sent with an `OPPTYPE`.
    ///
    /// The absence of an `OPPTYPE` leaves several `PictureOption`s unset that
    /// are still in force; higher-level decoder machinery is responsible for
    /// keeping track of options in force from previous pictures.
    pub has_opptype: bool,

    /// The intra-prediction mode in use, if any.
    pub picture_type: PictureTypeCode,

    /// Specifies the limits on motion vectors.
    ///
    /// Must be specified if and only if the `PictureOption` called
    /// `UnlimitedMotionVectors` is also enabled.
    pub motion_vector_range: Option<MotionVectorRange>,

    /// What slice-structured submodes are active.
    ///
    /// Must be specified if and only if the `PictureOption` called
    /// `SLICE_STRUCTURED` is also enabled.
    pub slice_submode: Option<SliceSubmode>,

    /// Which layer this picture is a member of.
    ///
    /// Only present if Temporal, SNR, and Spatial Scalability mode is enabled.
    pub scalability_layer: Option<ScalabilityLayer>,

    /// What backchannel signals is the encoder requesting from it's decoding
    /// partner.
    pub reference_picture_selection_mode: Option<ReferencePictureSelectionMode>,

    /// ITU-T Recommendation H.263 (01/2005) 5.1.14-5.1.15 `TRP`,`TRPI`
    ///
    /// Indicates the temporal reference of the picture to be used to
    /// reconstruct this picture. Must not be specified if this is an `IFrame`
    /// or `EiFrame`. For `BFrame`s, this field indicates the reference number
    /// of the forward-predicted reference frame. If not specified, intra
    /// prediction proceeds as if `REFERENCE_PICTURE_SELECTION` had not been
    /// enabled.
    pub prediction_reference: Option<u16>,

    /// ITU-T Recommendation H.263 (01/2005) 5.1.16 `BCI`
    ///
    /// This field stores any backchannel message requests sent by the encoder.
    /// This field may only be present if `REFERENCE_PICTURE_SELECTION` has
    /// been enabled.
    pub backchannel_message: Option<BackchannelMessage>,

    /// ITU-T Recommendation H.263 (01/2005) 5.1.18 `RPRP`
    ///
    /// Carries the parameters of the `REFERENCE_PICTURE_RESAMPLING` mode.
    pub reference_picture_resampling: Option<ReferencePictureResampling>,

    /// ITU-T Recommendation H.263 (01/2005) 5.1.19 `PQUANT`
    ///
    /// The quantizer factor to be used for this picture (unless otherwise
    /// overridden in a particular lower layer).
    pub quantizer: u8,

    /// ITU-T Recommendation H.263 (01/2005) 5.1.20-5.1.21 `CPM`, `PSBI`
    ///
    /// A number from 0 to 3 indicating which multipoint sub-bitstream this
    /// picture is a member of. If `None`, then the continuous presence
    /// multipoint feature is not enabled.
    pub multiplex_bitstream: Option<u8>,

    /// ITU-T Recommendation H.263 (01/2005) 5.1.22 `TRb`
    ///
    /// The number of non-transmitted frames to the B half of the current PB
    /// frame. This field should not be present if not using PB frames or their
    /// improved variety.
    pub pb_reference: Option<u8>,

    /// ITU-T Recommendation H.263 (01/2005) 5.1.23 `DBQUANT`
    ///
    /// The quantization factor used for the B block of a PB frame. This field
    /// should not be present if not using PB frames or their improved variety.
    pub pb_quantizer: Option<BPictureQuantizer>,

    /// ITU-T Recommendation H.263 (01/2005) 5.1.24 `PEI`
    ///
    /// Extra information bytes which may have been added to this picture.
    pub extra: Vec<u8>,
}

/// The default resolution options available in H.263.
///
/// The `CIF` refers to "Common Interchange Format", a video teleconferencing
/// resolution and framerate standard intended to be a halfway house between
/// analog PAL and NTSC video formats. It has the line rate of PAL, with the
/// frame rate of NTSC, and always encodes color as 4:2:0 YCbCr. It's digital
/// video resolution is 352x288 @ 30000/1001hz.
///
/// Most other `SourceFormat` variants are multiples of the CIF picture count.
/// Note that the multiples refer to total pixel count; i.e. a `FourCif` format
/// image is twice the width and height of a `FullCif` format image.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SourceFormat {
    /// 128x96 @ 30000/1001hz
    SubQcif,

    /// 176x144 @ 30000/1001hz
    QuarterCif,

    /// 352x288 @ 30000/1001hz
    FullCif,

    /// 704x576 @ 30000/1001hz
    FourCif,

    /// 1408x1152 @ 30000/1001hz
    SixteenCif,

    /// Reserved by H.264 spec. Does not appear to be in use.
    Reserved,

    /// A custom source format.
    Extended(CustomPictureFormat),
}

impl SourceFormat {
    /// Determine the source format's pixel width and height.
    ///
    /// Please note that these only determine the size of internal buffers
    /// necessary for decoding the picture. The actual shape of the user's
    /// presented video is determined by both this aspect ratio and the pixel
    /// aspect ratio of this source format.
    ///
    /// This function returns `None` if the source format is `Reserved`.
    pub fn into_width_and_height(self) -> Option<(u16, u16)> {
        match self {
            Self::SubQcif => Some((128, 96)),
            Self::QuarterCif => Some((176, 144)),
            Self::FullCif => Some((352, 288)),
            Self::FourCif => Some((704, 576)),
            Self::SixteenCif => Some((1408, 1152)),
            Self::Reserved => None,
            Self::Extended(cpf) => {
                Some((cpf.picture_width_indication, cpf.picture_height_indication))
            }
        }
    }
}

bitflags! {
    /// All H.263 options configured by `PTYPE` and `OPPTYPE`.
    ///
    /// Many of these options are specified in annexes to H.263 and are not
    /// required to be supported in all decoders. The meaning of each picture
    /// option should be referenced from ITU-T Recommendation H.263 (01/2005).
    ///
    /// Certain combinations of `PictureOption`s are mutually exclusive and
    /// using them together will result in errors in compliant decoders. Some
    /// `PictureTypeCode`s will also prohibit the use of certain
    /// `PictureOption`s.
    pub struct PictureOption : u32 {
        const USE_SPLIT_SCREEN = 0b1;
        const USE_DOCUMENT_CAMERA = 0b10;
        const RELEASE_FULL_PICTURE_FREEZE = 0b100;
        const UNRESTRICTED_MOTION_VECTORS = 0b1000;
        const SYNTAX_BASED_ARITHMETIC_CODING = 0b10000;
        const ADVANCED_PREDICTION = 0b100000;
        const ADVANCED_INTRA_CODING = 0b1000000;
        const DEBLOCKING_FILTER = 0b10000000;
        const SLICE_STRUCTURED = 0b100000000;
        const REFERENCE_PICTURE_SELECTION = 0b1000000000;
        const INDEPENDENT_SEGMENT_DECODING = 0b10000000000;
        const ALTERNATIVE_INTER_VLC = 0b100000000000;
        const MODIFIED_QUANTIZATION = 0b1000000000000;
        const REFERENCE_PICTURE_RESAMPLING = 0b10000000000000;
        const REDUCED_RESOLUTION_UPDATE = 0b100000000000000;
        const ROUNDING_TYPE_ONE = 0b1000000000000000;

        /// Advisory flag to request use of a deblocking filter.
        ///
        /// This flag is only set by Sorenson Spark bitstreams.
        const USE_DEBLOCKER = 0b10000000000000000;
    }
}

lazy_static! {
    /// The set of options only present in the `OPPTYPE` portion of the picture
    /// header.
    pub static ref OPPTYPE_OPTIONS: PictureOption =
        PictureOption::UNRESTRICTED_MOTION_VECTORS
            | PictureOption::SYNTAX_BASED_ARITHMETIC_CODING
            | PictureOption::ADVANCED_PREDICTION
            | PictureOption::ADVANCED_INTRA_CODING
            | PictureOption::DEBLOCKING_FILTER
            | PictureOption::SLICE_STRUCTURED
            | PictureOption::REFERENCE_PICTURE_SELECTION
            | PictureOption::INDEPENDENT_SEGMENT_DECODING
            | PictureOption::ALTERNATIVE_INTER_VLC
            | PictureOption::MODIFIED_QUANTIZATION;

    /// The set of options only present in the `MPPTYPE` portion of the picture
    /// header.
    pub static ref MPPTYPE_OPTIONS: PictureOption =
        PictureOption::REFERENCE_PICTURE_RESAMPLING
            | PictureOption::REDUCED_RESOLUTION_UPDATE
            | PictureOption::ROUNDING_TYPE_ONE;
}

/// All available picture types in H.263.
///
/// A picture type indicates what reference frames should be used, if any, to
/// decode the image.
///
/// Certain `PictureTypeCode`s will prohibit the use of particular
/// `PictureOption`s.
#[derive(Copy, Clone, Debug)]
pub enum PictureTypeCode {
    /// A full picture update that can be independently decoded.
    IFrame,

    /// A partial picture update that references a previously decoded frame.
    PFrame,

    /// PB frames.
    PbFrame,

    /// "Improved" PB frames.
    ImprovedPbFrame,

    /// A partial picture update that references up to two decoded frames, any
    /// of which may be future frames.
    BFrame,

    /// EI frames
    EiFrame,

    /// EP frames
    EpFrame,

    /// A reserved picture type code.
    ///
    /// The provided `u8` is the `MPPTYPE` that was reserved, anchored to the
    /// lowest significant bit of the `u8`.
    Reserved(u8),

    /// A partial picture update that references a previously decoded frame.
    ///
    /// This particular picture type has an additional stipulation: the encoder
    /// promises not to code frames that reference this one. The decoder is
    /// thus free to dispose of it after the fact.
    ///
    /// This picture type is exclusive to Sorenson Spark bitstreams.
    DisposablePFrame,
}

impl PictureTypeCode {
    /// Determine if this picture type is either kind of PB frame.
    pub fn is_any_pbframe(self) -> bool {
        matches!(self, Self::PbFrame) || matches!(self, Self::ImprovedPbFrame)
    }

    pub fn is_disposable(self) -> bool {
        matches!(self, Self::DisposablePFrame)
    }
}

/// ITU-T Recommendation H.263 (01/2005) 5.1.5-5.1.6 `CPFMT`, `EPAR`
///
/// This defines a "custom" picture format, outside of the standard CIF options.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CustomPictureFormat {
    /// The aspect ratio of a single pixel.
    pub pixel_aspect_ratio: PixelAspectRatio,

    /// The number of pixels per line.
    pub picture_width_indication: u16,

    /// The number of lines per image.
    pub picture_height_indication: u16,
}

/// The aspect ratio of dots on each line.
///
/// Pixel aspect ratio is a hangover from the world of analog video, where the
/// line rate was determined by CRT circuitry but you could divide up that line
/// by any regular clock you wanted. The number of pixels per line determined
/// the aspect ratio of the dots you generated on the fundamentally analog CRT
/// screen.
///
/// The pixel aspect ratio does not determine anything about the structure of
/// the video data. It only determines how it should be stretched to produce
/// the correct aspect ratio.
///
/// Most modern video formats should be `Square`. Legacy analog formats may be
/// stored in one of the `ParNN_NN` formats. A custom PAR may be indicated with
/// the `Extended` option.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PixelAspectRatio {
    /// 1:1 pixel aspect ratio. Most common on modern displays.
    Square,

    /// 12:11 pixel aspect ratio. Noted as "CIF for 4:3 Picture" in H.263.
    Par12_11,

    /// 10:11 pixel aspect ratio. Noted as "525-type for 4:3 Picture" in H.263.
    Par10_11,

    /// 16:11 pixel aspect ratio. Noted as "CIF stretched for 16:9 Picture" in
    /// H.263.
    Par16_11,

    /// 40:33 pixel aspect ratio. Noted as "525-type stretched for 16:9
    /// Picture" in H.263.
    Par40_33,

    /// One of the reserved PAR options.
    ///
    /// The provided `u8` is the `PAR` code that was reserved, anchored to the
    /// lowest significant bit of the `u8`.
    Reserved(u8),

    /// An extended/custom pixel aspect ratio.
    ///
    /// It is forbidden to have a zero width or height pixel.
    Extended { par_width: u8, par_height: u8 },
}

/// ITU-T Recommendation H.263 (01/2005) 5.1.7 `CPCFC`
///
/// The conversion between these factors and frame rate is as follows: Take
/// 1,800,000hz, and divide it by the effective divisor to produce a frame
/// rate. The effective divisor is `divisor` times either 1000 or 1001,
/// depending on the `times_1001` flag.
pub struct CustomPictureClock {
    /// Whether or not the divisor is multiplied by 1000 or 1001.
    ///
    /// `true` indicates 1001, whilst `false` indicates 1000.
    pub times_1001: bool,

    /// The divisor, itself stored divided by a constant factor (see
    /// `times_1001`.)
    pub divisor: u8,
}

/// ITU-T Recommendation H.263 (01/2005) 5.1.9 `UUI`
///
/// Indicates the new motion vector range limitations when
/// `UNRESTRICTED_MOTION_VECTORS` are enabled.
#[derive(Debug)]
pub enum MotionVectorRange {
    /// Motion vector component ranges are extended to limits that are
    /// prescribed in ITU-T Recommendation H.263 (01/2005) D.1 and D.2.
    Extended,

    /// Motion vector component ranges are only limited by the picture size.
    Unlimited,
}

bitflags! {
    /// ITU-T Recommendation H.263 (01/2005) 5.1.9 `SSS`
    ///
    /// Indicates slice configuration when slice-structured mode is enabled.
    pub struct SliceSubmode : u8 {
        /// Slices must be rectantular rather than free-running.
        const RECTANGULAR_SLICES = 0b1;

        /// Slices may be sent in arbitrary order.
        const ARBITRARY_ORDER = 0b10;
    }
}

/// ITU-T Recommendation H.263 (01/2005) 5.1.11-5.1.12 `ELNUM`, `RLNUM`
///
/// Only present if Temporal, SNR, and Spatial Scalability is enabled.
#[derive(Debug)]
pub struct ScalabilityLayer {
    /// The 4-bit enhancement layer index.
    pub enhancement: u8,

    /// The 4-bit reference layer index.
    ///
    /// If `None`, then this picture does not specify the reference layer for
    /// this layer. You should refer to previous pictures that do declare a
    /// reference layer in order to obtain that value in this case.
    pub reference: Option<u8>,
}

bitflags! {
    /// ITU-T Recommendation H.263 (01/2005) 5.1.13 `RPSMF`
    ///
    /// Indicates what backchannel messages the encoder would like out of it's
    /// decoding partner.
    pub struct ReferencePictureSelectionMode : u8 {
        const RESERVED = 0b1;
        const REQUEST_NEGATIVE_ACKNOWLEDGEMENT = 0b10;
        const REQUEST_ACKNOWLEDGEMENT = 0b100;
    }
}

/// ITU-T Recommendation H.263 (01/2005) N.4.2 `BCM`
///
/// Indicates backchannel information that a decoder of a (presumably live)
/// video stream is sending in response to an opposing video stream. It may be
/// presented to the encoder with a separate logical channel, or it may be
/// muxed into a video stream that the encoder is also expected to decode.
#[derive(Debug)]
pub struct BackchannelMessage {
    /// What message type is being back-channeled.
    message_type: BackchannelMessageType,

    /// Whether or not the backchanneler has reliable reference numbers to the
    /// opposing video stream. This being set to `Unreliable` indicates that
    /// the references in this message may not be correct.
    reliable: BackchannelReliability,

    /// The temporal reference of the picture being backchanneled.
    temporal_reference: u16,

    /// The enhancement layer being backchanneled, or `None` if no layer was
    /// specified.
    enhancement_layer: Option<u8>,

    /// The sub-bitstream number being backchanneled.
    sub_bitstream: Option<u8>,

    /// The GOB number or macroblock address being backchanneled.
    gob_macroblock_address: Option<u16>,

    /// The temporal reference being requested for retransmission (if NACK).
    requested_temporal_reference: Option<u16>,
}

/// ITU-T Recommendation H.263 (01/2005) N.4.2.1 `BT`
///
/// Indicates the backchanneler's decoding status of the opposing video stream.
#[derive(Debug)]
#[allow(dead_code)]
pub enum BackchannelMessageType {
    /// Positive acknowledgement of correct decoding of the opposing video
    /// stream.
    Acknowledge,

    /// Negative acknowledgement of erroneous or failed decoding of the
    /// opposing video stream.
    NegativeAcknowledge,

    /// Reserved message type.
    Reserved(u8),
}

/// ITU-T Recommendation H.263 (01/2005) N.4.2.2 `URF`
///
/// Whether or not the backchanneling decoder has reliable values for temporal
/// references, group-of-block numbers, or macroblock addresses.
#[derive(Debug)]
#[allow(dead_code)]
pub enum BackchannelReliability {
    Reliable,
    Unreliable,
}

/// ITU-T Recommendation H.263 (01/2005) P.2 `RPRP`
///
/// The parameters necessary for reference-picture resampling.
#[derive(Debug)]
pub struct ReferencePictureResampling {
    accuracy: WarpingDisplacementAccuracy,

    /// The eight warping parameters for reference picture resampling.
    ///
    /// Each parameter is encoded according to table `D.3` in H.263 (01/2005).
    /// This is a variable-length code whose decoded values max out at around
    /// 11 bits.
    warps: Option<[u16; 8]>,
}

/// ITU-T Recommendation H.263 (01/2005) P.2.1 `WDA`
#[derive(Debug)]
#[allow(dead_code)]
pub enum WarpingDisplacementAccuracy {
    /// Warping parameters are quantized to half-pixel accuracy.
    HalfPixel,

    /// Warping parameters are quantized to sixteenth-pixel accuracy.
    SixteenthPixel,
}

/// ITU-T Recommendation H.263 (01/2005), 5.1.23 `DBQUANT`
#[derive(Debug)]
pub enum BPictureQuantizer {
    FiveFourths,
    SixFourths,
    SevenFourths,
    EightFourths,
}

/// ITU-T Recommendation H.263 (01/2005), 5.2.x `GN`, `GSBI`, `GFID`, `GQUANT`
///
/// In an H.264-compliant bitstream, each picture is composed of one or more
/// groups of blocks. The first group of blocks is implied and *not*
/// transmitted in a compliant bitstream. Sorenson bitstreams treat all
/// pictures as a single group of blocks, and thus will not use this structure.
pub struct GroupOfBlocks {
    /// The GOB number.
    ///
    /// This number is never 0, as the picture header is also treated as if it
    /// were the first GOB header. Furthermore, this is limited to groups 1-17
    /// when standard picture source formats (CIF) are used, or 1-24 for custom
    /// picture formats. Higher group numbers are prohibited as they are used
    /// in slice-structured mode or end codes.
    pub group_number: u8,

    /// ITU-T Recommendation H.263 (01/2005) 5.2.4 `GSBI`
    ///
    /// A number from 0 to 3 indicating which multipoint sub-bitstream this
    /// group of blocks is a member of. If `None`, then the continuous presence
    /// multipoint feature is not enabled.
    pub multiplex_bitstream: Option<u8>,

    /// ITU-T Recommendation H.263 (01/2005) 5.2.5 `GFID`
    pub frame_id: u8,

    /// ITU-T Recommendation H.263 (01/2005) 5.2.6 `GQUANT`
    ///
    /// The quantizer factor to be used for this group of blocks until later
    /// changed by another GOB or macroblock.
    pub quantizer: u8,
}

/// ITU-T Recommendation H.263 (01/2005), 5.3 Macroblock layer
#[derive(Debug)]
pub enum Macroblock {
    /// Indicates a macroblock that isn't coded.
    ///
    /// This macroblock type is only valid outside of I-pictures, and indicates
    /// a macroblock which should be replaced with it's reference picture data.
    Uncoded,

    /// Indicates non-coding bits inserted to avoid a run of 16 consecutive
    /// zeroes.
    Stuffing,

    /// Indicates a coded macroblock containing picture data.
    Coded {
        /// The type of macroblock sent.
        mb_type: MacroblockType,

        /// The blocks within the macroblock that contain non-DC components.
        coded_block_pattern: CodedBlockPattern,

        /// The blocks within the macroblock that correspond to the B component
        /// of the B frame.
        coded_block_pattern_b: Option<CodedBlockPattern>,

        /// ITU-T Recommendation H.263 (01/2005) 5.3.6 `DQUANT`
        d_quantizer: Option<i8>,

        /// ITU-T Recommendation H.263 (01/2005) 5.3.7 `MVD`
        motion_vector: Option<MotionVector>,

        /// ITU-T Recommendation H.263 (01/2005) 5.3.8 `MVD2-4`
        addl_motion_vectors: Option<[MotionVector; 3]>,

        /// ITU-T Recommendation H.263 (01/2005) 5.3.9 `MVDB`
        motion_vectors_b: Option<[MotionVector; 4]>,
    },
}

impl Macroblock {
    /// Get the macroblock type of this macroblock.
    ///
    /// Returns `None` if this macroblock is uncoded or stuffing.
    pub fn macroblock_type(&self) -> Option<MacroblockType> {
        match self {
            Self::Uncoded => None,
            Self::Stuffing => None,
            Self::Coded { mb_type, .. } => Some(*mb_type),
        }
    }
}

/// ITU-T Recommendation H.263 (01/2005), 5.3.2 `MCBPC` (block-type half)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MacroblockType {
    /// Macroblock.
    Inter,

    /// Macroblock with quantizer delta.
    InterQ,

    /// Macroblock with motion vectors.
    Inter4V,

    /// Macroblock with `INTRADC` components.
    Intra,

    /// Macroblock with `INTRADC` components and quantizer delta.
    IntraQ,

    /// Macroblock with quantizer delta and motion vectors.
    Inter4Vq,
}

impl MacroblockType {
    /// Determine if this is an `INTER` macroblock.
    pub fn is_inter(self) -> bool {
        matches!(self, Self::Inter)
            || matches!(self, Self::InterQ)
            || matches!(self, Self::Inter4V)
            || matches!(self, Self::Inter4Vq)
    }

    /// Determine if this is an `INTRA` macroblock.
    pub fn is_intra(self) -> bool {
        matches!(self, Self::Intra) || matches!(self, Self::IntraQ)
    }

    /// Determine if this macroblock has four motion vectors.
    pub fn has_fourvec(self) -> bool {
        matches!(self, Self::Inter4V) || matches!(self, Self::Inter4Vq)
    }

    /// Determine if this macroblock has it's own quantizer.
    pub fn has_quantizer(self) -> bool {
        matches!(self, Self::InterQ)
            || matches!(self, Self::IntraQ)
            || matches!(self, Self::Inter4Vq)
    }
}

/// ITU-T Recommendation H.263 (01/2005), 5.3.2 `MCBPC`, 5.3.5 `CBPY`
///
/// Coded block pattern bits that indicate which blocks contain frequency
/// components to be coded for.
#[derive(Clone, Debug)]
pub struct CodedBlockPattern {
    pub codes_luma: [bool; 4],
    pub codes_chroma_b: bool,
    pub codes_chroma_r: bool,
}

/// Half-pixel motion vector components.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HalfPel(i16);

impl From<f32> for HalfPel {
    fn from(float: f32) -> Self {
        HalfPel((float * 2.0).floor() as i16)
    }
}

impl HalfPel {
    pub const STANDARD_RANGE: Self = Self(32);
    pub const EXTENDED_RANGE: Self = Self(64);
    pub const EXTENDED_RANGE_QUADCIF: Self = Self(128);
    pub const EXTENDED_RANGE_SIXTEENCIF: Self = Self(256);
    pub const EXTENDED_RANGE_BEYONDCIF: Self = Self(512);

    /// Construct a half-pel from some value that already contains half-pel
    /// units.
    pub fn from_unit(unit: i16) -> Self {
        HalfPel(unit)
    }

    pub fn zero() -> Self {
        Self(0)
    }

    /// Separate the half-pixel into lerp parameters.
    ///
    /// The first parameter indicates the offset of the top/left coordinate to
    /// sample from, and the second indicates if you also need to lerp with the
    /// pixel to the right or bottom.
    pub fn into_lerp_parameters(self) -> (i16, bool) {
        if self.0 % 2 == 0 {
            (self.0 / 2, false)
        } else if self < HalfPel::zero() {
            (self.0 / 2 - 1, true)
        } else {
            (self.0 / 2, true)
        }
    }

    /// Invert the HalfPel around the restricted MVD component range.
    ///
    /// For example, given a HalfPel decoded from the Vector column of H.263
    /// (2005/01) table 14, this gives you the equivalent entry from the
    /// Differences column of that table.
    pub fn invert(self) -> Self {
        match self.0.cmp(&0) {
            Ordering::Greater => Self(self.0 - 64),
            Ordering::Less => Self(self.0 + 64),
            Ordering::Equal => self,
        }
    }

    /// Determine if the half-pel is within the restricted MVD component range.
    pub fn is_mv_within_range(self, range: HalfPel) -> bool {
        -range.0 <= self.0 && self.0 < range.0
    }

    /// Determine if the half-pel is within the restricted MVD component range.
    pub fn is_predictor_within_range(self, range: HalfPel) -> bool {
        -range.0 < self.0 && self.0 <= range.0
    }

    /// Given the sum of four motion vectors, divide by eight and round to the
    /// nearest full pixel.
    ///
    /// It is expected that this `HalfPel` is actually the sum of four motion
    /// vectors.
    pub fn average_sum_of_mvs(self) -> Self {
        let whole = (self.0 >> 4) << 1; // div 8
        let frac = self.0 & 0x0F;

        match frac {
            0 | 1 | 2 => Self(whole),
            14 | 15 => Self(whole + 2),
            _ => Self(whole + 1),
        }
    }

    /// Given this and two other values, determine the median value and return
    /// it.
    pub fn median_of(self, mhs: Self, rhs: Self) -> Self {
        if self > mhs {
            if rhs > mhs {
                if rhs > self {
                    //rhs, self, mhs
                    self
                } else {
                    //self, rhs, mhs
                    rhs
                }
            } else {
                //self, mhs, rhs
                mhs
            }
        } else if mhs > rhs {
            if rhs > self {
                // mhs, rhs, self
                rhs
            } else {
                // mhs, self, rhs
                self
            }
        } else {
            // mhs, self
            mhs
        }
    }
}

impl Add<HalfPel> for HalfPel {
    type Output = HalfPel;

    fn add(self, rhs: Self) -> Self {
        HalfPel(self.0 + rhs.0)
    }
}

impl Div<i16> for HalfPel {
    type Output = HalfPel;

    fn div(self, rhs: i16) -> Self {
        HalfPel(self.0 / rhs)
    }
}

impl Neg for HalfPel {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

/// A motion vector consisting of X and Y components.
#[derive(Copy, Clone, Debug)]
pub struct MotionVector(HalfPel, HalfPel);

impl MotionVector {
    pub fn zero() -> Self {
        Self(HalfPel::zero(), HalfPel::zero())
    }

    pub fn into_lerp_parameters(self) -> ((i16, bool), (i16, bool)) {
        (self.0.into_lerp_parameters(), self.1.into_lerp_parameters())
    }

    pub fn average_sum_of_mvs(self) -> Self {
        Self(self.0.average_sum_of_mvs(), self.1.average_sum_of_mvs())
    }

    /// Given this and two other motion vectors, determine the median value of
    /// each component and return it.
    pub fn median_of(self, mhs: Self, rhs: Self) -> Self {
        MotionVector(
            self.0.median_of(mhs.0, rhs.0),
            self.1.median_of(mhs.1, rhs.1),
        )
    }
}

impl From<(HalfPel, HalfPel)> for MotionVector {
    fn from(vectors: (HalfPel, HalfPel)) -> Self {
        Self(vectors.0, vectors.1)
    }
}

impl From<MotionVector> for (HalfPel, HalfPel) {
    fn from(mv: MotionVector) -> Self {
        (mv.0, mv.1)
    }
}

impl Add<MotionVector> for MotionVector {
    type Output = MotionVector;

    fn add(self, rhs: Self) -> Self {
        MotionVector(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Div<i16> for MotionVector {
    type Output = MotionVector;

    fn div(self, rhs: i16) -> Self {
        MotionVector(self.0 / rhs, self.1 / rhs)
    }
}

/// ITU-T Recommendation H.263 (01/2005) 5.4 "Block layer"
///
/// A block is the most basic unit of picture coding. It consists of a number
/// of transform coefficients which are dequantized and then fed into an
/// inverse cosine transform. It can also be layered on top of existing frame
/// data, optionally transformed by a motion vector.
#[derive(PartialEq, Eq, Debug)]
pub struct Block {
    /// The DC component of the block, if present.
    pub intradc: Option<IntraDc>,

    /// All remaining block coefficients, stored as `TCOEF` events.
    pub tcoef: Vec<TCoefficient>,
}

/// ITU-T Recommendation H.263 (01/2005) 5.4.1 `INTRADC`
///
/// The DC coefficient for intra blocks is coded in a somewhat weird way; this
/// struct handles coding it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct IntraDc(u8);

impl IntraDc {
    /// Convert a fixed-level code u8 into an IntraDc value.
    ///
    /// This function yields `None` for values that are not valid FLC values
    /// as per Table 15/H.263.
    pub fn from_u8(value: u8) -> Option<Self> {
        if value == 0 || value == 128 {
            None
        } else {
            Some(IntraDc(value))
        }
    }

    /// Convert a reconstruction level into an IntraDc value.
    ///
    /// This function yields `None` for out-of-range or otherwise
    /// unrepresentable level constants.
    pub fn from_level(value: u16) -> Option<Self> {
        if (value & 0x07) != 0 || value > 2032 || value < 8 {
            return None;
        }

        if value == 1024 {
            return Some(IntraDc(0xFF));
        }

        Some(IntraDc((value >> 3) as u8))
    }

    /// Retrieve the reconstruction level of the DC component.
    pub fn into_level(self) -> i16 {
        if self.0 == 0xFF {
            1024
        } else {
            (self.0 as u16 as i16) << 3
        }
    }
}

/// ITU-T Recommendation H.263 (01/2005) 5.4.2 `TCOEF`
///
/// Represents an IDCT coefficient stored in quantized, run-length encoded
/// format. Trailing zeros are not coded; encoders should refrain from encoding
/// trailing zeroes and decoders should pad the decompressed block data with
/// zeroes.
#[derive(PartialEq, Eq, Debug)]
pub struct TCoefficient {
    /// Indicates if the `TCOEF` was or is to be encoded using the shorter,
    /// variable-length code (VLC) for coefficients.
    ///
    /// Not all coefficients can be encoded using the VLC.
    pub is_short: bool,

    /// The number of zero coefficients preceding this one.
    pub run: u8,

    /// The non-zero value at the end of the current run.
    ///
    /// In H.263 compliant bitstreams, `LEVEL` never exceeds 8 bits. In
    /// Sorenson Spark version 1 bitstreams, `LEVEL` is either 7 or 11 bits.
    pub level: i16,
}
