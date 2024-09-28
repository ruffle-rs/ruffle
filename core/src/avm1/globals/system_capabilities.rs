use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::system::SystemCapabilities;
use crate::avm1::object::Object;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{ScriptObject, Value};
use crate::string::{AvmString, StringContext};

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "supports64BitProcesses" => property(get_has_64_bit_support);
    "supports32BitProcesses" => property(get_has_32_bit_support);
    "isEmbeddedInAcrobat" => property(get_is_acrobat_embedded);
    "hasTLS" => property(get_has_tls);
    "cpuArchitecture" => property(get_cpu_architecture);
    "hasAccessibility" => property(get_has_accessibility);
    "hasAudio" => property(get_has_audio);
    "hasAudioEncoder" => property(get_has_audio_encoder);
    "hasEmbeddedVideo" => property(get_has_embedded_video);
    "hasIME" => property(get_has_ime);
    "hasMP3" => property(get_has_mp3);
    "hasPrinting" => property(get_has_printing);
    "hasScreenBroadcast" => property(get_has_screen_broadcast);
    "hasScreenPlayback" => property(get_has_screen_playback);
    "hasStreamingAudio" => property(get_has_streaming_audio);
    "hasStreamingVideo" => property(get_has_streaming_video);
    "hasVideoEncoder" => property(get_has_video_encoder);
    "isDebugger" => property(get_is_debugger);
    "avHardwareDisable" => property(get_is_av_hardware_disabled);
    "localFileReadDisable" => property(get_is_local_file_read_disabled);
    "windowlessDisable" => property(get_is_windowless_disabled);
    "language" => property(get_language);
    "manufacturer" => property(get_manufacturer);
    "os" => property(get_os_name);
    "pixelAspectRatio" => property(get_pixel_aspect_ratio);
    "playerType" => property(get_player_type);
    "screenColor" => property(get_screen_color);
    "screenDPI" => property(get_screen_dpi);
    "screenResolutionX" => property(get_screen_resolution_x);
    "screenResolutionY" => property(get_screen_resolution_y);
    "serverString" => property(get_server_string);
    "version" => property(get_version);
    "maxLevelIDC" => property(get_max_idc_level);
};

macro_rules! capabilities_func {
    ($func_name: ident, $capability: expr) => {
        pub fn $func_name<'gc>(
            activation: &mut Activation<'_, 'gc>,
            _this: Object<'gc>,
            _args: &[Value<'gc>],
        ) -> Result<Value<'gc>, Error<'gc>> {
            Ok(activation.context.system.has_capability($capability).into())
        }
    };
}

macro_rules! inverse_capabilities_func {
    ($func_name: ident, $capability: expr) => {
        pub fn $func_name<'gc>(
            activation: &mut Activation<'_, 'gc>,
            _this: Object<'gc>,
            _args: &[Value<'gc>],
        ) -> Result<Value<'gc>, Error<'gc>> {
            Ok((!activation.context.system.has_capability($capability)).into())
        }
    };
}

capabilities_func!(get_has_64_bit_support, SystemCapabilities::PROCESS_64_BIT);
capabilities_func!(get_has_32_bit_support, SystemCapabilities::PROCESS_32_BIT);
capabilities_func!(
    get_is_acrobat_embedded,
    SystemCapabilities::ACROBAT_EMBEDDED
);
capabilities_func!(get_has_tls, SystemCapabilities::TLS);
capabilities_func!(get_has_accessibility, SystemCapabilities::ACCESSIBILITY);
capabilities_func!(get_has_audio, SystemCapabilities::AUDIO);
capabilities_func!(get_has_audio_encoder, SystemCapabilities::AUDIO_ENCODER);
capabilities_func!(get_has_embedded_video, SystemCapabilities::EMBEDDED_VIDEO);

capabilities_func!(get_has_ime, SystemCapabilities::IME);
capabilities_func!(get_has_mp3, SystemCapabilities::MP3);
capabilities_func!(get_has_printing, SystemCapabilities::PRINTING);
capabilities_func!(
    get_has_screen_broadcast,
    SystemCapabilities::SCREEN_BROADCAST
);
capabilities_func!(get_has_screen_playback, SystemCapabilities::SCREEN_PLAYBACK);
capabilities_func!(get_has_streaming_audio, SystemCapabilities::STREAMING_AUDIO);
capabilities_func!(get_has_streaming_video, SystemCapabilities::STREAMING_VIDEO);
capabilities_func!(get_has_video_encoder, SystemCapabilities::VIDEO_ENCODER);
capabilities_func!(get_is_debugger, SystemCapabilities::DEBUGGER);
inverse_capabilities_func!(
    get_is_local_file_read_disabled,
    SystemCapabilities::LOCAL_FILE_READ
);
inverse_capabilities_func!(get_is_av_hardware_disabled, SystemCapabilities::AV_HARDWARE);
inverse_capabilities_func!(get_is_windowless_disabled, SystemCapabilities::WINDOW_LESS);

pub fn get_player_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new_utf8(
        activation.context.gc_context,
        activation.context.system.player_type.to_string(),
    )
    .into())
}

pub fn get_screen_color<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new_utf8(
        activation.context.gc_context,
        activation.context.system.screen_color.to_string(),
    )
    .into())
}

pub fn get_language<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new_utf8(
        activation.context.gc_context,
        activation
            .context
            .system
            .language
            .get_language_code(activation.context.avm1.player_version()),
    )
    .into())
}

pub fn get_screen_resolution_x<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let viewport_dimensions = activation.context.renderer.viewport_dimensions();
    // Viewport size is adjusted for HiDPI.
    let adjusted_width = f64::from(viewport_dimensions.width) / viewport_dimensions.scale_factor;
    Ok(adjusted_width.round().into())
}

pub fn get_screen_resolution_y<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let viewport_dimensions = activation.context.renderer.viewport_dimensions();
    // Viewport size is adjusted for HiDPI.
    let adjusted_height = f64::from(viewport_dimensions.height) / viewport_dimensions.scale_factor;
    Ok(adjusted_height.round().into())
}

pub fn get_pixel_aspect_ratio<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.system.pixel_aspect_ratio.into())
}

pub fn get_screen_dpi<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.system.dpi.into())
}

pub fn get_manufacturer<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new_utf8(
        activation.context.gc_context,
        activation
            .context
            .system
            .manufacturer
            .get_manufacturer_string(activation.context.avm1.player_version()),
    )
    .into())
}

pub fn get_os_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new_utf8(
        activation.context.gc_context,
        activation.context.system.os.to_string(),
    )
    .into())
}

pub fn get_version<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new_utf8(
        activation.context.gc_context,
        activation
            .context
            .system
            .get_version_string(activation.context.avm1),
    )
    .into())
}

pub fn get_server_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let server_string = activation
        .context
        .system
        .get_server_string(activation.context);
    Ok(AvmString::new_utf8(activation.context.gc_context, server_string).into())
}

pub fn get_cpu_architecture<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new_utf8(
        activation.context.gc_context,
        activation.context.system.cpu_architecture.to_string(),
    )
    .into())
}

pub fn get_max_idc_level<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new_utf8(
        activation.context.gc_context,
        &activation.context.system.idc_level,
    )
    .into())
}

pub fn create<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let capabilities = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(OBJECT_DECLS, context, capabilities, fn_proto);
    capabilities.into()
}
