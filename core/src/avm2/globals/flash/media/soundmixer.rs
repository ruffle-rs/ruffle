//! `flash.media.SoundMixer` builtin/prototype

use std::cell::RefMut;
use std::sync::Arc;

use crate::avm2::activation::Activation;
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::Object;
use crate::avm2::object::TObject;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::avm2_stub_getter;
use crate::display_object::SoundTransform;
use gc_arena::GcCell;
use once_cell::sync::Lazy;

/// Implements `flash.media.SoundMixer`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;
    }

    Ok(Value::Undefined)
}

/// Implements `flash.media.SoundMixer`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Implements `soundTransform`'s getter
///
/// This also implements `SimpleButton`'s `soundTransform` property, as per
/// Flash Player behavior.
pub fn get_sound_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let dobj_st = activation.context.global_sound_transform().clone();

    Ok(dobj_st.into_avm2_object(activation)?.into())
}

/// Implements `soundTransform`'s setter
///
/// This also implements `SimpleButton`'s `soundTransform` property, as per
/// Flash Player behavior.
pub fn set_sound_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let as3_st = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_object(activation)?;
    let dobj_st = SoundTransform::from_avm2_object(activation, as3_st)?;

    activation.context.set_global_sound_transform(dobj_st);

    Ok(Value::Undefined)
}

/// Implements `SoundMixer.stopAll`
pub fn stop_all<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.context.stop_all_sounds();

    Ok(Value::Undefined)
}

/// Implements `bufferTime`'s getter
pub fn buffer_time<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.audio_manager.stream_buffer_time().into())
}

/// Implements `bufferTime`'s setter
pub fn set_buffer_time<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let buffer_time = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_i32(activation)?;

    activation
        .context
        .audio_manager
        .set_stream_buffer_time(buffer_time);

    Ok(Value::Undefined)
}

/// `SoundMixer.areSoundsInaccessible`
pub fn are_sounds_inaccessible<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
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
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let arg0 = args[0].as_object().unwrap();
    let mut bytearray: RefMut<ByteArrayStorage> = arg0
        .as_bytearray_mut(activation.context.gc_context)
        .unwrap();
    let mut hist = activation.context.audio.get_sample_history();

    let fft = args.len() > 1 && args[1].coerce_to_boolean();
    let stretch = if args.len() > 2 {
        args[2].coerce_to_i32(activation)?
    } else {
        0
    };

    if fft {
        // Flash Player appears to do a 2048-long FFT with only the first 512 samples filled in...
        static FFT: Lazy<Arc<dyn realfft::RealToComplex<f32>>> =
            Lazy::new(|| realfft::RealFftPlanner::new().plan_fft_forward(2048));

        let fft = FFT.as_ref();

        let mut in_left = fft.make_input_vec();
        let mut in_right = fft.make_input_vec();

        for ((il, ir), h) in in_left
            .iter_mut()
            .zip(in_right.iter_mut())
            .zip(hist)
            .take(512)
        {
            *il = h[0];
            *ir = h[1];
        }

        let mut out_left = fft.make_output_vec();
        let mut out_right = fft.make_output_vec();

        // An error is only returned if any of the slices are the wrong size,
        // but they can't be, because the fft made them itself.
        let mut scratch = fft.make_scratch_vec();
        let _ = fft.process_with_scratch(&mut in_left, &mut out_left, &mut scratch);
        let _ = fft.process_with_scratch(&mut in_right, &mut out_right, &mut scratch);

        // This function was reverse-engineered with blood and tears.
        #[inline]
        fn postproc(x: f32) -> f32 {
            x.abs().ln().max(0.0) / 4.0
        }

        for (h, (ol, or)) in hist
            .iter_mut()
            .zip((out_left.iter()).zip(out_right.iter()))
            .take(512)
        {
            *h = [postproc(ol.re), postproc(or.re)];
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
            bytearray.write_float(sample[0])?;
            if bytearray.position() >= 1024 {
                break 'outer;
            }
        }
    }
    // Writing out the right channel values.
    'outer: for sample in &hist[..256] {
        for _ in 0..repeats {
            bytearray.write_float(sample[1])?;
            if bytearray.position() >= 2048 {
                break 'outer;
            }
        }
    }
    // The read head has to be rewound for AS.
    bytearray.set_position(0);
    Ok(Value::Undefined)
}

/// Construct `SoundMixer`'s class.
pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> GcCell<'gc, Class<'gc>> {
    let mc = activation.context.gc_context;
    let class = Class::new(
        QName::new(Namespace::package("flash.media", mc), "SoundMixer"),
        Some(Multiname::new(activation.avm2().public_namespace, "Object")),
        Method::from_builtin(instance_init, "<SoundMixer instance initializer>", mc),
        Method::from_builtin(class_init, "<SoundMixer class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED | ClassAttributes::FINAL);

    const PUBLIC_CLASS_PROPERTIES: &[(&str, Option<NativeMethodImpl>, Option<NativeMethodImpl>)] =
        &[
            (
                "soundTransform",
                Some(get_sound_transform),
                Some(set_sound_transform),
            ),
            ("bufferTime", Some(buffer_time), Some(set_buffer_time)),
        ];
    write.define_builtin_class_properties(
        mc,
        activation.avm2().public_namespace,
        PUBLIC_CLASS_PROPERTIES,
    );

    const PUBLIC_CLASS_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("stopAll", stop_all),
        ("areSoundsInaccessible", are_sounds_inaccessible),
        ("computeSpectrum", compute_spectrum),
    ];
    write.define_builtin_class_methods(
        mc,
        activation.avm2().public_namespace,
        PUBLIC_CLASS_METHODS,
    );

    class
}
