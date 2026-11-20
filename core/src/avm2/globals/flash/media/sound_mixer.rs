//! `flash.media.SoundMixer` builtin/prototype

use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::function::FunctionArgs;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2_stub_getter;
use crate::display_object::SoundTransform;
use rustfft::{Fft, FftDirection, algorithm::Radix4, num_complex::Complex};
use std::{ops::Deref, sync::LazyLock};

/// Implements `soundTransform`'s getter
///
/// This also implements `SimpleButton`'s `soundTransform` property, as per
/// Flash Player behavior.
pub fn get_sound_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let dobj_st = activation.context.global_sound_transform();

    Ok(dobj_st.into_avm2_object(activation)?.into())
}

/// Implements `soundTransform`'s setter
///
/// This also implements `SimpleButton`'s `soundTransform` property, as per
/// Flash Player behavior.
pub fn set_sound_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let as3_st = args.get_object(activation, 0, "sndTransform")?;
    let dobj_st = SoundTransform::from_avm2_object(as3_st);

    activation.context.set_global_sound_transform(dobj_st);

    Ok(Value::Undefined)
}

/// Implements `SoundMixer.stopAll`
pub fn stop_all<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    activation.context.stop_all_sounds();

    Ok(Value::Undefined)
}

/// Implements `bufferTime`'s getter
pub fn get_buffer_time<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.audio_manager.stream_buffer_time().into())
}

/// Implements `bufferTime`'s setter
pub fn set_buffer_time<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let buffer_time = args.get_i32(0);

    activation
        .context
        .audio_manager
        .set_stream_buffer_time(buffer_time);

    Ok(Value::Undefined)
}

/// `SoundMixer.areSoundsInaccessible`
pub fn are_sounds_inaccessible<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(
        activation,
        "flash.media.SoundMixer",
        "areSoundsInaccessible"
    );
    Ok(false.into())
}

/// Implements `SoundMixer.computeSpectrum`
pub fn compute_spectrum<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let arg0 = args.get_object(activation, 0, "sound")?;
    let mut bytearray = arg0.as_bytearray_mut().unwrap();
    let mut hist = activation.context.audio.get_sample_history();

    let fft = args.get_bool(1);
    let stretch = args.get_i32(2);

    if fft {
        // Flash Player appears to do a 2048-long FFT with only the first 512 samples filled in...
        // The input is real, so the first half of the output is the same as
        // that of a real FFT. A fixed size algorithm avoids instantiating
        // every algorithm that a planner could choose from.
        static FFT: LazyLock<Radix4<f32>> =
            LazyLock::new(|| Radix4::new(2048, FftDirection::Forward));

        let fft = FFT.deref();

        // The samples go in, and the FFT replaces them with the spectrum.
        let mut left = vec![Complex::default(); 2048];
        let mut right = vec![Complex::default(); 2048];

        for ((l, r), h) in left
            .iter_mut()
            .zip(right.iter_mut())
            .zip(hist.iter())
            .take(512)
        {
            l.re = h[0];
            r.re = h[1];
        }

        let mut scratch = vec![Complex::default(); fft.get_inplace_scratch_len()];
        fft.process_with_scratch(&mut left, &mut scratch);
        fft.process_with_scratch(&mut right, &mut scratch);

        // This function was reverse-engineered with blood and tears.
        #[inline]
        fn postproc(x: f32) -> f32 {
            x.abs().ln().max(0.0) / 4.0
        }

        for (h, (l, r)) in hist
            .iter_mut()
            .zip((left.iter()).zip(right.iter()))
            .take(512)
        {
            *h = [postproc(l.re), postproc(r.re)];
        }
    }

    // A stretch factor of 0 appears to be "special" in that it squishes the
    // 512 used input values (both with or without FFT) of each channel into
    // 256 by skipping every odd one.
    if stretch == 0 {
        for i in 0..256 {
            hist[i] = hist[2 * i];
        }
    }

    // Positive stretch factors simply repeat every value some number of times.
    // Stretch factors 1 and 2 appear to be identical (again both with or
    // without FFT). They simply use the first 256 values without repetition.
    let repeats = (stretch - 1).max(1);

    bytearray.set_length(256 * 2 * 4);
    bytearray.set_position(0);
    // Writing out the left channel values.
    'outer: for sample in &hist[..256] {
        for _ in 0..repeats {
            bytearray
                .write_float(sample[0])
                .map_err(|e| e.to_avm(activation))?;
            if bytearray.position() >= 1024 {
                break 'outer;
            }
        }
    }
    // Writing out the right channel values.
    'outer: for sample in &hist[..256] {
        for _ in 0..repeats {
            bytearray
                .write_float(sample[1])
                .map_err(|e| e.to_avm(activation))?;
            if bytearray.position() >= 2048 {
                break 'outer;
            }
        }
    }
    // The read head has to be rewound for AS.
    bytearray.set_position(0);
    Ok(Value::Undefined)
}
