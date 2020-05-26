use crate::avm1::object::Object;
use crate::avm1::property::Attribute::{DontDelete, DontEnum, ReadOnly};
use crate::avm1::return_value::ReturnValue;
use crate::avm1::value::Value::{Bool, Undefined};
use crate::avm1::{Avm1, Error, ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use gc_arena::MutationContext;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use crate::avm1::globals::system_capabilities::Language::English;

#[allow(dead_code)]
enum Language {
    Czech,
    Danish,
    Dutch,
    English,
    Finnish,
    French,
    German,
    Hungarian,
    Italian,
    Japanese,
    Korean,
    Norwegian,
    Unknown,
    Polish,
    Portuguese,
    Russian,
    SimplifiedChinese,
    Spanish,
    Swedish,
    TraditionalChinese,
    Turkish
}

impl Language {
    fn get_language_code(&self) -> &str {
        return match self {
            Language::Czech => "cs",
            Language::Danish => "da",
            Language::Dutch => "nl",
            Language::English => {
                // TODO: return "en-US" for player_version < 7
                "en"
            }
            Language::Finnish => "fi",
            Language::French => "fr",
            Language::German => "de",
            Language::Hungarian => "hu",
            Language::Italian => "it",
            Language::Japanese => "ja",
            Language::Korean => "ko",
            Language::Norwegian => "no",
            Language::Unknown => "xu",
            Language::Polish => "pl",
            Language::Portuguese => "pt",
            Language::Russian => "ru",
            Language::SimplifiedChinese => "zh-CN",
            Language::Spanish => "es",
            Language::Swedish => "sv",
            Language::TraditionalChinese => "zh-TW",
            Language::Turkish => "tr"
        }
    }
}

pub fn create<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let capabilities = ScriptObject::object(gc_context, proto);

    capabilities.define_value(
        gc_context,
        "avHardwareDisable",
        true.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "hasAccessibility",
        false.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "hasAudio",
        true.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "hasAudioEncoder",
        false.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "hasEmbeddedVideo",
        false.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "hasIME",
        true.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "hasMP3",
        true.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "hasPrinting",
        false.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "hasScreenBroadcast",
        false.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "hasScreenPlayback",
        false.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "hasStreamingAudio",
        false.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "hasStreamingVideo",
        false.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "hasVideoEncoder",
        false.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "isDebugger",
        false.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    // TODO: note for fp <7 this should be the locale on windows and the ui lang for >= 7
    capabilities.define_value(
        gc_context,
        "language",
        English.get_language_code().into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "localFileReadDisable",
        true.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "manufacturer",
        "Macromedia Linux".into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "os",
        "Linux".into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "pixelAspectRatio",
        1.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "playerType",
        "StandAlone".into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "screenColor",
        "color".into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "screenDPI",
        false.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "screenResolutionX",
        1920.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "screenResolutionY",
        1080.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "serverString",
        "MP3=t&AE=f".into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.define_value(
        gc_context,
        "version",
        "WIN 8,0,0,0".into(),
        DontDelete | ReadOnly | DontEnum,
    );

    capabilities.into()
}
