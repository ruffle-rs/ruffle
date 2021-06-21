//! Picture-layer decoder

use crate::decoder::DecoderOption;
use crate::error::{Error, Result};
use crate::parser::reader::H263Reader;
use crate::types::{
    BPictureQuantizer, BackchannelMessage, CustomPictureClock, CustomPictureFormat,
    MotionVectorRange, Picture, PictureOption, PictureTypeCode, PixelAspectRatio,
    ReferencePictureResampling, ReferencePictureSelectionMode, ScalabilityLayer, SliceSubmode,
    SourceFormat,
};
use std::io::Read;

/// The information imparted by a `PTYPE` record.
///
/// If the optional portion of this type is `None`, that signals that a
/// `PLUSPTYPE` immediately follows the `PTYPE` record.
pub type PType = (PictureOption, Option<(SourceFormat, PictureTypeCode)>);

/// Decodes the first 8 bits of `PTYPE`.
fn decode_ptype<R>(reader: &mut H263Reader<R>) -> Result<PType>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        let mut options = PictureOption::empty();

        let high_ptype_bits = reader.read_u8()?;
        if high_ptype_bits & 0xC0 != 0x80 {
            return Err(Error::InvalidPType);
        }

        if high_ptype_bits & 0x20 != 0 {
            options |= PictureOption::USE_SPLIT_SCREEN;
        }

        if high_ptype_bits & 0x10 != 0 {
            options |= PictureOption::USE_DOCUMENT_CAMERA;
        }

        if high_ptype_bits & 0x08 != 0 {
            options |= PictureOption::RELEASE_FULL_PICTURE_FREEZE;
        }

        let source_format = match high_ptype_bits & 0x07 {
            0 => return Err(Error::InvalidPType),
            1 => SourceFormat::SubQcif,
            2 => SourceFormat::QuarterCif,
            3 => SourceFormat::FullCif,
            4 => SourceFormat::FourCif,
            5 => SourceFormat::SixteenCif,
            6 => SourceFormat::Reserved,
            _ => return Ok((options, None)),
        };

        let low_ptype_bits: u8 = reader.read_bits(5)?;
        let mut r#type = if low_ptype_bits & 0x10 != 0 {
            PictureTypeCode::IFrame
        } else {
            PictureTypeCode::PFrame
        };

        if low_ptype_bits & 0x08 != 0 {
            options |= PictureOption::UNRESTRICTED_MOTION_VECTORS;
        }

        if low_ptype_bits & 0x04 != 0 {
            options |= PictureOption::SYNTAX_BASED_ARITHMETIC_CODING;
        }

        if low_ptype_bits & 0x02 != 0 {
            options |= PictureOption::ADVANCED_PREDICTION;
        }

        if low_ptype_bits & 0x01 != 0 {
            r#type = PictureTypeCode::PbFrame;
        }

        Ok((options, Some((source_format, r#type))))
    })
}

bitflags! {
    /// Indicates which fields follow `PLUSPTYPE`.
    ///
    /// A field is only listed in here if the H.263 spec mentions the
    /// requirement that `UFEP` equal 001. Otherwise, the existence of a
    /// follower can be determined by the set of `PictureOption`s returned in
    /// the `PlusPType`.
    pub struct PlusPTypeFollower: u8 {
        const HAS_CUSTOM_FORMAT = 0b1;
        const HAS_CUSTOM_CLOCK = 0b10;
        const HAS_MOTION_VECTOR_RANGE = 0b100;
        const HAS_SLICE_STRUCTURED_SUBMODE = 0b1000;
        const HAS_REFERENCE_LAYER_NUMBER = 0b10000;
        const HAS_REFERENCE_PICTURE_SELECTION_MODE = 0b100000;
    }
}

/// The information imparted by a `PLUSPTYPE` record.
///
/// `SourceFormat` is optional and will be `None` either if the record did not
/// specify a `SourceFormat` or if it specified a custom one. To determine if
/// one needs to be parsed, read the `PlusPTypeFollower`s, which indicate
/// additional records which follow this one in the bitstream.
///
/// The `bool` indicates if `OPPTYPE` was present in the `PLUSPTYPE` record.
pub type PlusPType = (
    PictureOption,
    Option<SourceFormat>,
    PictureTypeCode,
    PlusPTypeFollower,
    bool,
);

lazy_static! {
    /// The set of picture options defined by an `OPPTYPE` record.
    ///
    /// If a picture does not contain an `OPPTYPE`, then all of these options
    /// will be carried forward from the previous picture's options.
    static ref OPPTYPE_OPTIONS: PictureOption = PictureOption::UNRESTRICTED_MOTION_VECTORS
        | PictureOption::SYNTAX_BASED_ARITHMETIC_CODING
        | PictureOption::ADVANCED_PREDICTION
        | PictureOption::ADVANCED_INTRA_CODING
        | PictureOption::DEBLOCKING_FILTER
        | PictureOption::SLICE_STRUCTURED
        | PictureOption::REFERENCE_PICTURE_SELECTION
        | PictureOption::INDEPENDENT_SEGMENT_DECODING
        | PictureOption::ALTERNATIVE_INTER_VLC
        | PictureOption::MODIFIED_QUANTIZATION;
}

/// Attempts to read a `PLUSPTYPE` record from the bitstream.
///
/// The set of previous picture options are used to carry forward previously-
/// enabled options in the case where the `PLUSPTYPE` does not change them.
fn decode_plusptype<R>(
    reader: &mut H263Reader<R>,
    decoder_options: DecoderOption,
    previous_picture_options: PictureOption,
) -> Result<PlusPType>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        let ufep: u8 = reader.read_bits(3)?;
        let has_opptype = match ufep {
            0 => false,
            1 => true,
            _ => return Err(Error::InvalidPlusPType),
        };

        let mut options = PictureOption::empty();
        let mut followers = PlusPTypeFollower::empty();
        let mut source_format = None;

        if has_opptype {
            let opptype: u32 = reader.read_bits(18)?;

            // OPPTYPE should end in bits 1000 as per H.263 5.1.4.2
            if (opptype & 0xF) != 0x8 {
                return Err(Error::InvalidPlusPType);
            }

            source_format = match (opptype & 0x38000) >> 15 {
                0 => Some(SourceFormat::Reserved),
                1 => Some(SourceFormat::SubQcif),
                2 => Some(SourceFormat::QuarterCif),
                3 => Some(SourceFormat::FullCif),
                4 => Some(SourceFormat::FourCif),
                5 => Some(SourceFormat::SixteenCif),
                6 => {
                    followers |= PlusPTypeFollower::HAS_CUSTOM_FORMAT;

                    None
                }
                _ => Some(SourceFormat::Reserved),
            };

            if opptype & 0x04000 != 0 {
                followers |= PlusPTypeFollower::HAS_CUSTOM_CLOCK;
            }

            if opptype & 0x02000 != 0 {
                options |= PictureOption::UNRESTRICTED_MOTION_VECTORS;
                followers |= PlusPTypeFollower::HAS_MOTION_VECTOR_RANGE;
            }

            if opptype & 0x01000 != 0 {
                options |= PictureOption::SYNTAX_BASED_ARITHMETIC_CODING;
            }

            if opptype & 0x00800 != 0 {
                options |= PictureOption::ADVANCED_PREDICTION;
            }

            if opptype & 0x00400 != 0 {
                options |= PictureOption::ADVANCED_INTRA_CODING;
            }

            if opptype & 0x00200 != 0 {
                options |= PictureOption::DEBLOCKING_FILTER;
            }

            if opptype & 0x00100 != 0 {
                options |= PictureOption::SLICE_STRUCTURED;
                followers |= PlusPTypeFollower::HAS_SLICE_STRUCTURED_SUBMODE;
            }

            if opptype & 0x00080 != 0 {
                options |= PictureOption::REFERENCE_PICTURE_SELECTION;
                followers |= PlusPTypeFollower::HAS_REFERENCE_PICTURE_SELECTION_MODE;
            }

            if opptype & 0x00040 != 0 {
                options |= PictureOption::INDEPENDENT_SEGMENT_DECODING;
            }

            if opptype & 0x00020 != 0 {
                options |= PictureOption::ALTERNATIVE_INTER_VLC;
            }

            if opptype & 0x00010 != 0 {
                options |= PictureOption::MODIFIED_QUANTIZATION;
            }

            if decoder_options.contains(DecoderOption::USE_SCALABILITY_MODE) {
                followers |= PlusPTypeFollower::HAS_REFERENCE_LAYER_NUMBER;
            }
        } else {
            options |= previous_picture_options & *OPPTYPE_OPTIONS;
        }

        let mpptype: u16 = reader.read_bits(9)?;

        // MPPTYPE should end in bits 001 as per H.263 5.1.4.3
        if mpptype & 0x007 != 0x1 {
            return Err(Error::InvalidPlusPType);
        }

        let picture_type = match (mpptype & 0x1C0) >> 6 {
            0 => PictureTypeCode::IFrame,
            1 => PictureTypeCode::PFrame,
            2 => PictureTypeCode::ImprovedPbFrame,
            3 => PictureTypeCode::BFrame,
            4 => PictureTypeCode::EiFrame,
            5 => PictureTypeCode::EpFrame,
            r => PictureTypeCode::Reserved(r as u8),
        };

        if mpptype & 0x020 != 0 {
            options |= PictureOption::REFERENCE_PICTURE_RESAMPLING;
        }

        if mpptype & 0x010 != 0 {
            options |= PictureOption::REDUCED_RESOLUTION_UPDATE;
        }

        if mpptype & 0x008 != 0 {
            options |= PictureOption::ROUNDING_TYPE_ONE;
        }

        Ok((options, source_format, picture_type, followers, has_opptype))
    })
}

type SorensonPType = (SourceFormat, PictureTypeCode, PictureOption);

/// Attempts to read a Sorenson-equivalent PTYPE from the bitstream.
fn decode_sorenson_ptype<R>(reader: &mut H263Reader<R>) -> Result<SorensonPType>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        let (mut source_format, bit_count) = match reader.read_bits(3)? {
            0 => (None, 8),
            1 => (None, 16),
            2 => (Some(SourceFormat::FullCif), 0),
            3 => (Some(SourceFormat::QuarterCif), 0),
            4 => (Some(SourceFormat::SubQcif), 0),
            5 => (
                Some(SourceFormat::Extended(CustomPictureFormat {
                    pixel_aspect_ratio: PixelAspectRatio::Square,
                    picture_width_indication: 320,
                    picture_height_indication: 240,
                })),
                0,
            ),
            6 => (
                Some(SourceFormat::Extended(CustomPictureFormat {
                    pixel_aspect_ratio: PixelAspectRatio::Square,
                    picture_width_indication: 160,
                    picture_height_indication: 120,
                })),
                0,
            ),
            _ => (Some(SourceFormat::Reserved), 0),
        };

        if source_format.is_none() {
            let custom_width = reader.read_bits(bit_count)?;
            let custom_height = reader.read_bits(bit_count)?;

            source_format = Some(SourceFormat::Extended(CustomPictureFormat {
                pixel_aspect_ratio: PixelAspectRatio::Square,
                picture_width_indication: custom_width,
                picture_height_indication: custom_height,
            }));
        }

        let picture_type = match reader.read_bits(2)? {
            0 => PictureTypeCode::IFrame,
            1 => PictureTypeCode::PFrame,
            2 => PictureTypeCode::DisposablePFrame,
            r => PictureTypeCode::Reserved(r),
        };

        let mut options = PictureOption::empty();

        if reader.read_bits::<u8>(1)? == 1 {
            options |= PictureOption::USE_DEBLOCKER;
        }

        Ok((source_format.unwrap(), picture_type, options))
    })
}

/// Attempts to read `CPM` and `PSBI` records from the bitstream.
///
/// The placement of this record changes based on whether or not a `PLUSPTYPE`
/// is present in the bitstream. If it is present, then this function should
/// be called immediately after parsing it. Otherwise, this function should be
/// called after parsing `PQUANT`.
fn decode_cpm_and_psbi<R>(reader: &mut H263Reader<R>) -> Result<Option<u8>>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        if reader.read_bits::<u8>(1)? != 0 {
            Ok(Some(reader.read_bits::<u8>(2)?))
        } else {
            Ok(None)
        }
    })
}

/// Attempts to read `CPFMT` from the bitstream.
fn decode_cpfmt<R>(reader: &mut H263Reader<R>) -> Result<CustomPictureFormat>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        let cpfmt: u32 = reader.read_bits(23)?;

        if cpfmt & 0x000200 == 0 {
            return Err(Error::PictureFormatInvalid);
        }

        let pixel_aspect_ratio = match (cpfmt & 0x780000) >> 19 {
            0 => return Err(Error::PictureFormatInvalid),
            1 => PixelAspectRatio::Square,
            2 => PixelAspectRatio::Par12_11,
            3 => PixelAspectRatio::Par10_11,
            4 => PixelAspectRatio::Par16_11,
            5 => PixelAspectRatio::Par40_33,
            15 => {
                let par_width = reader.read_u8()?;
                let par_height = reader.read_u8()?;

                if par_width == 0 || par_height == 0 {
                    return Err(Error::PictureFormatInvalid);
                }

                PixelAspectRatio::Extended {
                    par_width,
                    par_height,
                }
            }
            r => PixelAspectRatio::Reserved(r as u8),
        };

        let picture_width_indication = (((cpfmt & 0x07FC00) >> 10) as u16 + 1) * 4;
        let picture_height_indication = ((cpfmt & 0x0000FF) as u16) * 4;

        Ok(CustomPictureFormat {
            pixel_aspect_ratio,
            picture_width_indication,
            picture_height_indication,
        })
    })
}

/// Attempts to read `CPCFC` from the bitstream.
fn decode_cpcfc<R>(reader: &mut H263Reader<R>) -> Result<CustomPictureClock>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        let cpcfc = reader.read_u8()?;

        Ok(CustomPictureClock {
            times_1001: cpcfc & 0x80 != 0,
            divisor: cpcfc & 0x7F,
        })
    })
}

/// Attempts to read `UUI` from the bitstream.
fn decode_uui<R>(reader: &mut H263Reader<R>) -> Result<MotionVectorRange>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        let is_limited: u8 = reader.read_bits(1)?;
        if is_limited == 1 {
            return Ok(MotionVectorRange::Extended);
        }

        let is_unlimited: u8 = reader.read_bits(1)?;
        if is_unlimited == 1 {
            return Ok(MotionVectorRange::Unlimited);
        }

        Err(Error::InvalidBitstream)
    })
}

/// Attempts to read `SSS` from the bitstream.
fn decode_sss<R>(reader: &mut H263Reader<R>) -> Result<SliceSubmode>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        let mut sss = SliceSubmode::empty();
        let sss_bits: u8 = reader.read_bits(2)?;

        if sss_bits & 0x01 != 0 {
            sss |= SliceSubmode::RECTANGULAR_SLICES;
        }

        if sss_bits & 0x02 != 0 {
            sss |= SliceSubmode::ARBITRARY_ORDER;
        }

        Ok(sss)
    })
}

/// Attempts to read `ELNUM` and `RLNUM` from the bitstream.
fn decode_elnum_rlnum<R>(
    reader: &mut H263Reader<R>,
    followers: PlusPTypeFollower,
) -> Result<ScalabilityLayer>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        let enhancement = reader.read_bits(4)?;
        let reference = if followers.contains(PlusPTypeFollower::HAS_REFERENCE_LAYER_NUMBER) {
            Some(reader.read_bits(4)?)
        } else {
            None
        };

        Ok(ScalabilityLayer {
            enhancement,
            reference,
        })
    })
}

/// Attempts to read `RPSMF` from the bitstream.
fn decode_rpsmf<R>(reader: &mut H263Reader<R>) -> Result<ReferencePictureSelectionMode>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        let mut rpsmf = ReferencePictureSelectionMode::empty();
        let rpsmf_bits: u8 = reader.read_bits(3)?;

        if rpsmf_bits & 0x4 == 0 {
            rpsmf |= ReferencePictureSelectionMode::RESERVED;
        }

        if rpsmf_bits & 0x2 != 0 {
            rpsmf |= ReferencePictureSelectionMode::REQUEST_NEGATIVE_ACKNOWLEDGEMENT;
        }

        if rpsmf_bits & 0x1 != 0 {
            rpsmf |= ReferencePictureSelectionMode::REQUEST_ACKNOWLEDGEMENT;
        }

        Ok(rpsmf)
    })
}

/// Attempts to read `TRPI` and `TRP` from the bitstream.
fn decode_trpi<R>(reader: &mut H263Reader<R>) -> Result<Option<u16>>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        let trpi: u8 = reader.read_bits(1)?;

        if trpi == 1 {
            let trp: u16 = reader.read_bits(10)?;

            Ok(Some(trp))
        } else {
            Ok(None)
        }
    })
}

/// Attempts to read `BCI` and `BCM` from the bitstream.
fn decode_bcm<R>(reader: &mut H263Reader<R>) -> Result<Option<BackchannelMessage>>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        let bci: u8 = reader.read_bits(1)?;

        if bci == 1 {
            Err(Error::UnimplementedDecoding)
        } else {
            let not_bci: u8 = reader.read_bits(1)?;

            if not_bci == 1 {
                Ok(None)
            } else {
                // BCI must be `1` or `01`
                Err(Error::InvalidBitstream)
            }
        }
    })
}

/// Attempts to read `RPRP` from the bitstream.
fn decode_rprp<R>(reader: &mut H263Reader<R>) -> Result<Option<ReferencePictureResampling>>
where
    R: Read,
{
    reader.with_transaction(|_reader| Err(Error::UnimplementedDecoding))
}

/// Attempts to read `TRB` from the bitstream.
fn decode_trb<R>(reader: &mut H263Reader<R>, has_custom_pclk: bool) -> Result<u8>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        if has_custom_pclk {
            reader.read_bits::<u8>(5)
        } else {
            reader.read_bits::<u8>(3)
        }
    })
}

/// Attempts to read `DBQUANT` from the bitstream.
fn decode_dbquant<R>(reader: &mut H263Reader<R>) -> Result<BPictureQuantizer>
where
    R: Read,
{
    reader.with_transaction(|reader| match reader.read_bits::<u8>(2)? {
        0 => Ok(BPictureQuantizer::FiveFourths),
        1 => Ok(BPictureQuantizer::SixFourths),
        2 => Ok(BPictureQuantizer::SevenFourths),
        3 => Ok(BPictureQuantizer::EightFourths),
        _ => Err(Error::InternalDecoderError),
    })
}

/// Attempts to read the `PSUPP` block from the bitstream as another embedded
/// bitstream.
fn decode_pei<R>(reader: &mut H263Reader<R>) -> Result<Vec<u8>>
where
    R: Read,
{
    reader.with_transaction(|reader| {
        let mut data = Vec::new();

        loop {
            let has_pei: u8 = reader.read_bits(1)?;
            if has_pei == 1 {
                data.push(reader.read_u8()?);
            } else {
                break;
            }
        }

        Ok(data)
    })
}

/// Attempts to read a picture record from an H.263 bitstream.
///
/// If no valid start code could be found in the bitstream, this function will
/// raise an error. If it is currently at the start of a GOB instead of a
/// Picture, then it will yield `None`, signalling that the current data should
/// be parsed as a GOB.
///
/// The set of `DecoderOptions` allows configuring certain information about
/// the decoding process that cannot be determined by decoding the bitstream
/// itself.
///
/// `previous_picture_options` is the set of options that were enabled by the
/// last decoded picture. If this is the first decoded picture in the
/// bitstream, then this should be an empty set.
pub fn decode_picture<R>(
    reader: &mut H263Reader<R>,
    decoder_options: DecoderOption,
    previous_picture: Option<&Picture>,
) -> Result<Option<Picture>>
where
    R: Read,
{
    reader.with_transaction_union(|reader| {
        let skipped_bits = reader
            .recognize_start_code(false)?
            .ok_or(Error::MiddleOfBitstream)?;

        reader.skip_bits(17 + skipped_bits)?;

        let gob_id = reader.read_bits(5)?;

        if decoder_options.contains(DecoderOption::SORENSON_SPARK_BITSTREAM) {
            let temporal_reference = reader.read_u8()? as u16;
            let (source_format, picture_type, options) = decode_sorenson_ptype(reader)?;
            let quantizer: u8 = reader.read_bits(5)?;
            let extra = decode_pei(reader)?;

            return Ok(Some(Picture {
                //Sorenson abuses the GOB ID as a version field.
                version: Some(gob_id),
                temporal_reference,
                format: Some(source_format),
                options,
                has_plusptype: false,
                has_opptype: false,
                picture_type,
                quantizer,
                extra,

                //Sorenson is always unlimited
                motion_vector_range: Some(MotionVectorRange::Unlimited),

                //Here's a bunch more modes Sorenson doesn't use.
                slice_submode: None,
                scalability_layer: None,
                reference_picture_selection_mode: None,
                prediction_reference: None,
                backchannel_message: None,
                reference_picture_resampling: None,
                multiplex_bitstream: None,
                pb_reference: None,
                pb_quantizer: None,
            }));
        } else if gob_id != 0 {
            return Ok(None);
        }

        let low_tr = reader.read_u8()?;
        let (mut options, maybe_format_and_type) = decode_ptype(reader)?;
        let mut multiplex_bitstream = None;
        let (mut format, picture_type, followers, has_plusptype, has_opptype) =
            match maybe_format_and_type {
                Some((format, picture_type)) => (
                    Some(format),
                    picture_type,
                    PlusPTypeFollower::empty(),
                    false,
                    false,
                ),
                None => {
                    let (extra_options, maybe_format, picture_type, followers, has_opptype) =
                        decode_plusptype(
                            reader,
                            decoder_options,
                            previous_picture
                                .map(|p| p.options)
                                .unwrap_or_else(PictureOption::empty),
                        )?;

                    options |= extra_options;

                    multiplex_bitstream = Some(decode_cpm_and_psbi(reader)?);

                    (maybe_format, picture_type, followers, true, has_opptype)
                }
            };

        //TODO: H.263 5.1.4.4-6 indicate a number of semantic restrictions on
        //picture options, modes, and followers. We should be inspecting our
        //set of options and raising an error if they're incorrect at this
        //time.

        //TODO: Some pictures don't restate their previous format, but the
        //contents of the picture rely on if the format has changed. We need
        //`decode_picture` to be able to look up previous picture headers
        //somehow.

        if followers.contains(PlusPTypeFollower::HAS_CUSTOM_FORMAT) {
            format = Some(SourceFormat::Extended(decode_cpfmt(reader)?));
        }

        let picture_clock = if followers.contains(PlusPTypeFollower::HAS_CUSTOM_CLOCK) {
            Some(decode_cpcfc(reader)?)
        } else {
            None
        };

        let temporal_reference = if picture_clock.is_some() {
            let high_tr = reader.read_bits::<u16>(2)? << 8;

            high_tr | low_tr as u16
        } else {
            low_tr as u16
        };

        let motion_vector_range = if followers.contains(PlusPTypeFollower::HAS_MOTION_VECTOR_RANGE)
        {
            Some(decode_uui(reader)?)
        } else {
            None
        };

        let slice_submode = if followers.contains(PlusPTypeFollower::HAS_SLICE_STRUCTURED_SUBMODE) {
            Some(decode_sss(reader)?)
        } else {
            None
        };

        let scalability_layer = if decoder_options.contains(DecoderOption::USE_SCALABILITY_MODE) {
            Some(decode_elnum_rlnum(reader, followers)?)
        } else {
            None
        };

        let reference_picture_selection_mode =
            if followers.contains(PlusPTypeFollower::HAS_REFERENCE_PICTURE_SELECTION_MODE) {
                Some(decode_rpsmf(reader)?)
            } else {
                None
            };

        let prediction_reference = if options.contains(PictureOption::REFERENCE_PICTURE_SELECTION) {
            decode_trpi(reader)?
        } else {
            None
        };

        let backchannel_message = if options.contains(PictureOption::REFERENCE_PICTURE_SELECTION) {
            decode_bcm(reader)?
        } else {
            None
        };

        //TODO: this should be checking against the reference picture to see if we need RPRP
        let reference_picture_resampling = if options
            .contains(PictureOption::REFERENCE_PICTURE_RESAMPLING)
            || previous_picture
                .map(|p| p.format != format)
                .unwrap_or(false)
        {
            decode_rprp(reader)?
        } else {
            None
        };

        let quantizer: u8 = reader.read_bits(5)?;

        if multiplex_bitstream.is_none() {
            multiplex_bitstream = Some(decode_cpm_and_psbi(reader)?);
        }
        let multiplex_bitstream = multiplex_bitstream.unwrap();

        //TODO: This needs to know the picture clock, which has the usual
        //reference picture thing I mentioned before in the last TODO
        let (pb_reference, pb_quantizer) = if matches!(
            picture_type,
            PictureTypeCode::PbFrame | PictureTypeCode::ImprovedPbFrame
        ) {
            (
                Some(decode_trb(reader, picture_clock.is_some())?),
                Some(decode_dbquant(reader)?),
            )
        } else {
            (None, None)
        };

        let extra = decode_pei(reader)?;

        Ok(Some(Picture {
            version: None,
            temporal_reference,
            format,
            options,
            has_plusptype,
            has_opptype,
            picture_type,
            motion_vector_range,
            slice_submode,
            scalability_layer,
            reference_picture_selection_mode,
            prediction_reference,
            backchannel_message,
            reference_picture_resampling,
            quantizer,
            multiplex_bitstream,
            pb_reference,
            pb_quantizer,
            extra,
        }))
    })
}
