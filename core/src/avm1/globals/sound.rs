//! AVM1 Sound object
//! TODO: Sound position, transform, loadSound

use std::cell::Cell;
use std::fmt;

use gc_arena::{Collect, Gc, Mutation};
use ruffle_macros::istr;

use crate::avm1::activation::Activation;
use crate::avm1::clamp::Clamp;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{NativeObject, Object, ScriptObject, TObject, Value};
use crate::backend::audio::{SoundHandle, SoundInstanceHandle};
use crate::backend::navigator::Request;
use crate::character::Character;
use crate::display_object::{DisplayObject, SoundTransform, TDisplayObject};
use crate::string::StringContext;
use crate::{avm1_stub, avm_warn};

/// A `Sound` object that is tied to a sound from the `AudioBackend``.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct Sound<'gc>(Gc<'gc, SoundData<'gc>>);

#[derive(Collect)]
#[collect(no_drop)]
struct SoundData<'gc> {
    /// The sound that is attached to this object.
    sound: Cell<Option<SoundHandle>>,

    /// The instance of the last played sound on this object.
    sound_instance: Cell<Option<SoundInstanceHandle>>,

    /// Sounds in AVM1 are tied to a specific movie clip.
    owner: Option<DisplayObject<'gc>>,

    /// Position of the last playing sound in milliseconds.
    position: Cell<u32>,

    /// Duration of the currently attached sound in milliseconds.
    duration: Cell<Option<u32>>,

    /// Whether this sound is an external streaming MP3.
    /// This will be true if `Sound.loadSound` was called with `isStreaming` of `true`.
    /// A streaming sound can only have a single active instance.
    is_streaming: Cell<bool>,
}

impl fmt::Debug for Sound<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Sound")
            .field("ptr", &Gc::as_ptr(self.0))
            .field("sound", &self.0.sound.get())
            .field("sound_instance", &self.0.sound_instance.get())
            .field("owner", &self.0.owner)
            .finish()
    }
}

impl<'gc> Sound<'gc> {
    pub fn empty(mc: &Mutation<'gc>, owner: Option<DisplayObject<'gc>>) -> Sound<'gc> {
        Sound(Gc::new(
            mc,
            SoundData {
                sound: Cell::new(None),
                sound_instance: Cell::new(None),
                owner,
                position: Cell::new(0),
                duration: Cell::new(None),
                is_streaming: Cell::new(false),
            },
        ))
    }

    pub fn duration(self) -> Option<u32> {
        self.0.duration.get()
    }

    pub fn set_duration(self, duration: Option<u32>) {
        self.0.duration.set(duration);
    }

    pub fn sound(self) -> Option<SoundHandle> {
        self.0.sound.get()
    }

    pub fn set_sound(self, sound: Option<SoundHandle>) {
        self.0.sound.set(sound);
    }

    pub fn sound_instance(self) -> Option<SoundInstanceHandle> {
        self.0.sound_instance.get()
    }

    pub fn set_sound_instance(self, sound_instance: Option<SoundInstanceHandle>) {
        self.0.sound_instance.set(sound_instance);
    }

    pub fn owner(self) -> Option<DisplayObject<'gc>> {
        self.0.owner
    }

    pub fn position(self) -> u32 {
        self.0.position.get()
    }

    pub fn set_position(self, position: u32) {
        self.0.position.set(position);
    }

    pub fn is_streaming(self) -> bool {
        self.0.is_streaming.get()
    }

    pub fn set_is_streaming(self, is_streaming: bool) {
        self.0.is_streaming.set(is_streaming);
    }
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "attachSound" => method(attach_sound; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "duration" => property(duration; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getDuration" => method(duration; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setDuration" => method(set_duration; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "id3" => method(id3; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getBytesLoaded" => method(get_bytes_loaded; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getBytesTotal" => method(get_bytes_total; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getPan" => method(get_pan; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getTransform" => method(get_transform; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getVolume" => method(get_volume; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "loadSound" => method(load_sound; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "position" => property(position; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setPan" => method(set_pan; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setTransform" => method(set_transform; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setVolume" => method(set_volume; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "start" => method(start; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "stop" => method(stop; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

/// Implements `Sound`
pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // 1st parameter is the movie clip that "owns" all sounds started by this object.
    // `Sound.setTransform`, `Sound.stop`, etc. will affect all sounds owned by this clip.
    let owner = if let Some(owner) = args.get(0) {
        let start_clip = activation.target_clip_or_root();
        activation.resolve_target_display_object(start_clip, *owner, false)?
    } else {
        None
    };

    let sound = Sound::empty(activation.gc(), owner);
    this.set_native(activation.gc(), NativeObject::Sound(sound));

    Ok(this.into())
}

pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(context, Some(proto));
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    object.into()
}

fn attach_sound<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args.get(0).unwrap_or(&Value::Undefined);
    if let NativeObject::Sound(sound) = this.native() {
        let name = name.coerce_to_string(activation)?;
        let movie = sound
            .owner()
            .unwrap_or_else(|| activation.base_clip().avm1_root())
            .movie();
        if let Some((_, Character::Sound(sound_handle))) = activation
            .context
            .library
            .library_for_movie_mut(movie)
            .character_by_export_name(name)
        {
            sound.set_sound(Some(*sound_handle));
            sound.set_is_streaming(false);
            sound.set_duration(
                activation
                    .context
                    .audio
                    .get_sound_duration(*sound_handle)
                    .map(|d| d.round() as u32),
            );
            sound.set_position(0);
        } else {
            avm_warn!(activation, "Sound.attachSound: Sound '{}' not found", name);
        }
    } else {
        avm_warn!(activation, "Sound.attachSound: this is not a Sound");
    }
    Ok(Value::Undefined)
}

fn duration<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: Sound.duration was only added in SWFv6, but it is not version gated.
    // Return undefined for player <6 if we ever add player version emulation.
    if let NativeObject::Sound(sound) = this.native() {
        return Ok(sound.duration().map_or(Value::Undefined, |d| d.into()));
    } else {
        avm_warn!(activation, "Sound.duration: this is not a Sound");
    }
    Ok(Value::Undefined)
}

fn set_duration<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

fn get_bytes_loaded<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.swf_version() >= 6 {
        avm1_stub!(activation, "Sound", "getBytesLoaded");
        Ok(1.into())
    } else {
        Ok(Value::Undefined)
    }
}

fn get_bytes_total<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.swf_version() >= 6 {
        avm1_stub!(activation, "Sound", "getBytesTotal");
        Ok(1.into())
    } else {
        Ok(Value::Undefined)
    }
}

fn get_pan<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::Sound(sound) = this.native() {
        let transform = sound
            .owner()
            .map(|owner| owner.base().sound_transform().clone())
            .unwrap_or_else(|| activation.context.global_sound_transform().clone());
        Ok(transform.pan().into())
    } else {
        Ok(Value::Undefined)
    }
}

fn get_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::Sound(sound) = this.native() {
        let transform = sound
            .owner()
            .map(|owner| owner.base().sound_transform().clone())
            .unwrap_or_else(|| activation.context.global_sound_transform().clone());

        let obj = ScriptObject::new(
            &activation.context.strings,
            Some(activation.context.avm1.prototypes().object),
        );
        // Surprisingly `lr` means "right-to-left" and `rl` means "left-to-right".
        obj.set(istr!("ll"), transform.left_to_left.into(), activation)?;
        obj.set(istr!("lr"), transform.right_to_left.into(), activation)?;
        obj.set(istr!("rl"), transform.left_to_right.into(), activation)?;
        obj.set(istr!("rr"), transform.right_to_right.into(), activation)?;
        Ok(obj.into())
    } else {
        Ok(Value::Undefined)
    }
}

fn get_volume<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::Sound(sound) = this.native() {
        let transform = sound
            .owner()
            .map(|owner| owner.base().sound_transform().clone())
            .unwrap_or_else(|| activation.context.global_sound_transform().clone());
        Ok(transform.volume.into())
    } else {
        Ok(Value::Undefined)
    }
}

fn id3<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.swf_version() >= 6 {
        avm1_stub!(activation, "Sound", "id3");
    }
    Ok(Value::Undefined)
}

fn load_sound<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::Sound(sound) = this.native() {
        if let Some(url) = args.get(0) {
            let url = url.coerce_to_string(activation)?;
            let is_streaming = args
                .get(1)
                .unwrap_or(&Value::Undefined)
                .as_bool(activation.swf_version());
            if is_streaming {
                // Streaming MP3s can only have a single active instance.
                // (Previous `attachSound` instances will continue to play.)
                if let Some(sound_instance) = sound.sound_instance() {
                    activation.context.stop_sound(sound_instance);
                }
            }
            sound.set_is_streaming(is_streaming);
            let future = activation.context.load_manager.load_sound_avm1(
                activation.context.player.clone(),
                this,
                Request::get(url.to_utf8_lossy().into_owned()),
                is_streaming,
            );
            activation.context.navigator.spawn_future(future);
        }
    }
    Ok(Value::Undefined)
}

fn position<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: Sound.position was only added in SWFv6, but it is not version gated.
    // Return undefined for player <6 if we ever add player version emulation.
    if let NativeObject::Sound(sound) = this.native() {
        if sound.sound().is_some() {
            return Ok(sound.position().into());
        }
    } else {
        avm_warn!(activation, "Sound.position: this is not a Sound");
    }
    Ok(Value::Undefined)
}

fn set_pan<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let pan = args
        .get(0)
        .unwrap_or(&0.into())
        .coerce_to_f64(activation)?
        .clamp_to_i32();
    if let NativeObject::Sound(sound) = this.native() {
        if let Some(owner) = sound.owner() {
            let mut transform = owner.base().sound_transform().clone();
            transform.set_pan(pan);
            owner.set_sound_transform(activation.context, transform);
        } else {
            let mut transform = activation.context.global_sound_transform().clone();
            transform.set_pan(pan);
            activation.context.set_global_sound_transform(transform);
        }
    }

    Ok(Value::Undefined)
}

fn set_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let obj = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);

    if let NativeObject::Sound(sound) = this.native() {
        let mut transform = if let Some(owner) = sound.owner() {
            owner.base().sound_transform().clone()
        } else {
            activation.context.global_sound_transform().clone()
        };

        if obj.has_own_property(activation, istr!("ll")) {
            transform.left_to_left = obj
                .get(istr!("ll"), activation)?
                .coerce_to_i32(activation)?;
        }
        // Surprisingly `lr` means "right-to-left" and `rl` means "left-to-right".
        if obj.has_own_property(activation, istr!("rl")) {
            transform.left_to_right = obj
                .get(istr!("rl"), activation)?
                .coerce_to_i32(activation)?;
        }
        if obj.has_own_property(activation, istr!("lr")) {
            transform.right_to_left = obj
                .get(istr!("lr"), activation)?
                .coerce_to_i32(activation)?;
        }
        if obj.has_own_property(activation, istr!("rr")) {
            transform.right_to_right = obj
                .get(istr!("rr"), activation)?
                .coerce_to_i32(activation)?;
        }

        if let Some(owner) = sound.owner() {
            owner.set_sound_transform(activation.context, transform);
        } else {
            activation.context.set_global_sound_transform(transform);
        };
    }
    Ok(Value::Undefined)
}

fn set_volume<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let volume = args
        .get(0)
        .unwrap_or(&0.into())
        .coerce_to_f64(activation)?
        .clamp_to_i32();
    if let NativeObject::Sound(sound) = this.native() {
        if let Some(owner) = sound.owner() {
            let transform = SoundTransform {
                volume,
                ..*owner.base().sound_transform()
            };
            owner.set_sound_transform(activation.context, transform);
        } else {
            let transform = SoundTransform {
                volume,
                ..*activation.context.global_sound_transform()
            };
            activation.context.set_global_sound_transform(transform);
        }
    }

    Ok(Value::Undefined)
}

pub fn start<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let start_offset = args.get(0).unwrap_or(&0.into()).coerce_to_f64(activation)?;
    let loops = args.get(1).unwrap_or(&1.into()).coerce_to_f64(activation)?;

    // TODO: Handle loops > u16::MAX.
    let loops = (loops as u16).max(1);

    use swf::{SoundEvent, SoundInfo};
    if let NativeObject::Sound(sound) = this.native() {
        if let Some(sound_handle) = sound.sound() {
            if sound.is_streaming() {
                // Streaming MP3s can only have a single active instance.
                if let Some(sound_instance) = sound.sound_instance() {
                    activation.context.stop_sound(sound_instance);
                }
            }
            let sound_instance = activation.context.start_sound(
                sound_handle,
                &SoundInfo {
                    event: SoundEvent::Start,
                    in_sample: if start_offset > 0.0 {
                        Some((start_offset * 44100.0) as u32)
                    } else {
                        None
                    },
                    out_sample: None,
                    num_loops: loops,
                    envelope: None,
                },
                sound.owner(),
                Some(this),
            );
            if sound_instance.is_some() {
                sound.set_sound_instance(sound_instance);
            }
        } else {
            avm_warn!(activation, "Sound.start: No sound is attached");
        }
    } else {
        avm_warn!(activation, "Sound.start: Invalid sound");
    }

    Ok(Value::Undefined)
}

fn stop<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::Sound(sound) = this.native() {
        if let Some(name) = args.get(0) {
            // Usage 1: Stop all instances of a particular sound, using the name parameter.
            let name = name.coerce_to_string(activation)?;
            let movie = sound
                .owner()
                .unwrap_or_else(|| activation.base_clip().avm1_root())
                .movie();
            if let Some((_, Character::Sound(sound))) = activation
                .context
                .library
                .library_for_movie_mut(movie)
                .character_by_export_name(name)
            {
                // Stop all sounds with the given name.
                let sound = *sound;
                activation.context.stop_sounds_with_handle(sound);
            } else {
                avm_warn!(activation, "Sound.stop: Sound '{}' not found", name);
            }
        } else if let Some(owner) = sound.owner() {
            // Usage 2: Stop all sound running within a given clip.
            activation.context.stop_sounds_with_display_object(owner);
            sound.set_sound_instance(None);
        } else {
            // Usage 3: If there is no owner and no name, this call acts like `stopAllSounds()`.
            activation.context.stop_all_sounds();
        }
    } else {
        avm_warn!(activation, "Sound.stop: this is not a Sound");
    }

    Ok(Value::Undefined)
}
