use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::globals::system::SystemCapabilities;
use crate::avm1::object::Object;
use crate::avm1::property::Attribute;
use crate::avm1::{AvmString, ScriptObject, TObject, Value};
use gc_arena::MutationContext;

macro_rules! capabilities_func {
    ($func_name: ident, $capability: expr) => {
        pub fn $func_name<'gc>(
            activation: &mut Activation<'_, 'gc, '_>,
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
            activation: &mut Activation<'_, 'gc, '_>,
            _this: Object<'gc>,
            _args: &[Value<'gc>],
        ) -> Result<Value<'gc>, Error<'gc>> {
            Ok((!activation.context.system.has_capability($capability)).into())
        }
    };
}

macro_rules! capabilities_prop {
    ($gc_ctx: expr, $capabilities: expr, $fn_proto: ident, $($name:expr => $func:expr),* ) => {{
        $(
            $capabilities.add_property(
                $gc_ctx,
                $name,
                FunctionObject::function($gc_ctx, Executable::Native($func), Some($fn_proto), $fn_proto),
                None,
                Attribute::empty()
            );
        )*
    }};
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
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new(
        activation.context.gc_context,
        activation.context.system.player_type.to_string(),
    )
    .into())
}

pub fn get_screen_color<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new(
        activation.context.gc_context,
        activation.context.system.screen_color.to_string(),
    )
    .into())
}

pub fn get_language<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new(
        activation.context.gc_context,
        activation
            .context
            .system
            .language
            .get_language_code(activation.context.avm1.player_version)
            .to_string(),
    )
    .into())
}

pub fn get_screen_resolution_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.system.screen_resolution.0.into())
}

pub fn get_screen_resolution_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.system.screen_resolution.1.into())
}

pub fn get_pixel_aspect_ratio<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.system.aspect_ratio.into())
}

pub fn get_screen_dpi<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.system.dpi.into())
}

pub fn get_manufacturer<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new(
        activation.context.gc_context,
        activation
            .context
            .system
            .manufacturer
            .get_manufacturer_string(activation.context.avm1.player_version),
    )
    .into())
}

pub fn get_os_name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new(
        activation.context.gc_context,
        activation.context.system.os.to_string(),
    )
    .into())
}

pub fn get_version<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new(
        activation.context.gc_context,
        activation
            .context
            .system
            .get_version_string(activation.context.avm1),
    )
    .into())
}

pub fn get_server_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let server_string = activation
        .context
        .system
        .get_server_string(activation.context.avm1);
    Ok(AvmString::new(activation.context.gc_context, server_string).into())
}

pub fn get_cpu_architecture<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new(
        activation.context.gc_context,
        activation.context.system.cpu_architecture.to_string(),
    )
    .into())
}

pub fn get_max_idc_level<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new(
        activation.context.gc_context,
        activation.context.system.idc_level.clone(),
    )
    .into())
}

pub fn create<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let capabilities = ScriptObject::object(gc_context, proto);

    capabilities_prop!(gc_context, capabilities, fn_proto,
        "supports64BitProcesses" => get_has_64_bit_support,
        "supports32BitProcesses" => get_has_32_bit_support,
        "isEmbeddedInAcrobat" => get_is_acrobat_embedded,
        "hasTLS" => get_has_tls,
        "cpuArchitecture" => get_cpu_architecture,
        "hasAccessibility" => get_has_accessibility,
        "hasAudio" => get_has_audio,
        "hasAudioEncoder" => get_has_audio_encoder,
        "hasEmbeddedVideo" => get_has_embedded_video,
        "hasIME" => get_has_ime,
        "hasMP3" => get_has_mp3,
        "hasPrinting" => get_has_printing,
        "hasScreenBroadcast" => get_has_screen_broadcast,
        "hasScreenPlayback" => get_has_screen_playback,
        "hasStreamingAudio" => get_has_streaming_audio,
        "hasStreamingVideo" => get_has_streaming_video,
        "hasVideoEncoder" => get_has_video_encoder,
        "isDebugger" => get_is_debugger,
        "avHardwareDisable" => get_is_av_hardware_disabled,
        "localFileReadDisable" => get_is_local_file_read_disabled,
        "windowlessDisable" => get_is_windowless_disabled,
        "language" => get_language,
        "manufacturer" => get_manufacturer,
        "os" => get_os_name,
        "pixelAspectRatio" => get_pixel_aspect_ratio,
        "playerType"=>get_player_type,
        "screenColor" => get_screen_color,
        "screenDPI" => get_screen_dpi,
        "screenResolutionX" => get_screen_resolution_x,
        "screenResolutionY" => get_screen_resolution_y,
        "serverString" => get_server_string,
        "version" => get_version,
        "maxLevelIDC" => get_max_idc_level
    );

    capabilities.into()
}
