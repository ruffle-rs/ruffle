//! AVM1 Sound object
//! TODO: Sound position, transform, loadSound

use std::cell::Cell;
use std::fmt;

use gc_arena::barrier::unlock;
use gc_arena::{Collect, Gc, Mutation, RefLock};
use ruffle_macros::istr;

use crate::avm1::activation::Activation;
use crate::avm1::clamp::Clamp;
use crate::avm1::error::Error;
use crate::avm1::function::FunctionObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{NativeObject, Object, Value};
use crate::backend::audio::{SoundHandle, SoundInstanceHandle};
use crate::backend::navigator::Request;
use crate::character::Character;
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, SoundTransform, TDisplayObject};
use crate::string::StringContext;
use crate::{avm1_stub, avm_warn};

#[derive(Debug, Collect)]
#[collect(no_drop)]
struct QueuedPlay<'gc> {
    sound_object: Object<'gc>,
    start_offset: f64,
    loops: u16,
}

#[derive(Debug, Collect)]
#[collect(no_drop)]
enum SoundState<'gc> {
    /// Empty sound object, no sound playback allowed.
    Empty,

    /// Sound is loading, plays can be queued.
    Loading { queued_plays: Vec<QueuedPlay<'gc>> },

    /// Sound is loaded, plays can be started immadiately.
    Loaded {
        /// The sound that is attached to this object.
        #[collect(require_static)]
        sound: SoundHandle,
    },
}

/// A `Sound` object that is tied to a sound from the `AudioBackend``.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct Sound<'gc>(Gc<'gc, SoundData<'gc>>);

#[derive(Collect)]
#[collect(no_drop)]
struct SoundData<'gc> {
    state: RefLock<SoundState<'gc>>,

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
            .field("state", &self.0.state.borrow())
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
                state: RefLock::new(SoundState::Empty),
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
        if let SoundState::Loaded { sound } = *self.0.state.borrow() {
            Some(sound)
        } else {
            None
        }
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

    fn play(self, play: QueuedPlay<'gc>, context: &mut UpdateContext<'gc>) {
        let write = Gc::write(context.gc(), self.0);
        let sound_handle = match &mut *unlock!(write, SoundData, state).borrow_mut() {
            SoundState::Empty => {
                tracing::warn!("Ignoring Sound playback, because it's not loaded.");
                return;
            }
            SoundState::Loading { queued_plays } => {
                queued_plays.push(play);
                return;
            }
            SoundState::Loaded { sound } => *sound,
        };

        if self.is_streaming() {
            // Streaming MP3s can only have a single active instance.
            if let Some(sound_instance) = self.sound_instance() {
                context.stop_sound(sound_instance);
            }
        }
        let sound_instance = context.start_sound(
            sound_handle,
            &swf::SoundInfo {
                event: swf::SoundEvent::Start,
                in_sample: if play.start_offset > 0.0 {
                    Some((play.start_offset * 44100.0) as u32)
                } else {
                    None
                },
                out_sample: None,
                num_loops: play.loops,
                envelope: None,
            },
            self.owner(),
            Some(play.sound_object),
        );
        if sound_instance.is_some() {
            self.set_sound_instance(sound_instance);
        }
    }

    fn set_is_loading(self, context: &mut UpdateContext<'gc>) {
        // All queued plays are discarded at this point.
        let new_data = SoundState::Loading {
            queued_plays: Vec::new(),
        };
        unlock!(Gc::write(context.gc(), self.0), SoundData, state).replace(new_data);
    }

    pub fn load_sound(self, sound: SoundHandle, context: &mut UpdateContext<'gc>) {
        let new_data = SoundState::Loaded { sound };
        let write = Gc::write(context.gc(), self.0);
        let old_data = unlock!(write, SoundData, state).replace(new_data);

        if let SoundState::Loading { queued_plays } = old_data {
            for play in queued_plays {
                self.play(play, context);
            }
        }
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
fn constructor<'gc>(
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
    let object = Object::new(context, Some(proto));
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    object
}

pub fn create_constructor<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    FunctionObject::constructor(context, constructor, None, fn_proto, proto)
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
            sound.load_sound(sound_handle, activation.context);
            sound.set_is_streaming(false);
            sound.set_duration(
                activation
                    .context
                    .audio
                    .get_sound_duration(sound_handle)
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
            .map(|owner| owner.base().sound_transform())
            .unwrap_or_else(|| *activation.context.global_sound_transform());
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
            .map(|owner| owner.base().sound_transform())
            .unwrap_or_else(|| *activation.context.global_sound_transform());

        let obj = Object::new(
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
            .map(|owner| owner.base().sound_transform())
            .unwrap_or_else(|| *activation.context.global_sound_transform());
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
            sound.set_is_loading(activation.context);
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
            let transform = owner.base().sound_transform();
            owner.set_sound_transform(activation.context, transform.with_pan(pan));
        } else {
            let transform = activation.context.global_sound_transform();
            activation
                .context
                .set_global_sound_transform(transform.with_pan(pan));
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
            owner.base().sound_transform()
        } else {
            *activation.context.global_sound_transform()
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
                ..owner.base().sound_transform()
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

    if let NativeObject::Sound(sound) = this.native() {
        let play = QueuedPlay {
            sound_object: this,
            start_offset,
            loops,
        };
        sound.play(play, activation.context);
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
