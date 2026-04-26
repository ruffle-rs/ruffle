//! AVM1 Sound object

use std::cell::Cell;
use std::fmt;
use std::io::Cursor;

use gc_arena::barrier::unlock;
use gc_arena::{Collect, Gc, Mutation, RefLock};
use id3::Tag;
use ruffle_macros::istr;

use crate::avm1::activation::Activation;
use crate::avm1::clamp::Clamp;
use crate::avm1::error::Error;
use crate::avm1::function::FunctionObject;
use crate::avm1::property_decl::{DeclContext, PropertyOrder, StaticDeclarations, SystemClass};
use crate::avm1::{ArrayBuilder, Attribute, ExecutionReason, NativeObject, Object, Value};
use crate::backend::audio::{SoundHandle, SoundInstanceHandle};
use crate::backend::navigator::Request;
use crate::character::Character;
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, SoundTransform, TDisplayObject};
use crate::string::AvmString;
use crate::{avm_warn, avm1_stub};

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

    /// Sound is loaded, plays can be started immediately.
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

    /// The target path for the display object that will "own" this sound.
    /// Whether this resolves to a valid display object or not at the time of calling it
    /// will affect the functionality of most methods; see `has_valid_owner`.
    target: Option<AvmString<'gc>>,

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
            .field("target", &self.0.target)
            .finish()
    }
}

impl<'gc> Sound<'gc> {
    pub fn empty(mc: &Mutation<'gc>, target: Option<AvmString<'gc>>) -> Sound<'gc> {
        Sound(Gc::new(
            mc,
            SoundData {
                state: RefLock::new(SoundState::Empty),
                sound_instance: Cell::new(None),
                target,
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

    pub fn owner(self, activation: &mut Activation<'_, 'gc>) -> Option<DisplayObject<'gc>> {
        if let Some(target) = self.0.target {
            let start_clip = activation.target_clip_or_root();
            activation
                .resolve_target_display_object(start_clip, Value::String(target), false)
                .ok()?
        } else {
            None
        }
    }

    pub fn use_global_sound(self) -> bool {
        self.0.target.is_none()
    }

    /// Returns `true` if this sound is attached to the global sound
    /// (initial target was `Null` or `Undefined`), or if the target
    /// *currently* resolves to a valid display object.
    pub fn has_valid_owner(self, activation: &mut Activation<'_, 'gc>) -> bool {
        self.use_global_sound() || self.owner(activation).is_some()
    }

    /// Gets the sound transform for this sound.
    /// If this sound is set to use global sound, then the
    /// global sound transform will be returned. Otherwise,
    /// it will try to resolve the owner and get its sound
    /// transform. If it can't, then `None` is returned.
    pub fn sound_transform(self, activation: &mut Activation<'_, 'gc>) -> Option<SoundTransform> {
        if self.use_global_sound() {
            Some(*activation.context.global_sound_transform())
        } else if let Some(owner) = self.owner(activation) {
            Some(owner.base().sound_transform())
        } else {
            None
        }
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

    fn play(self, play: QueuedPlay<'gc>, activation: &mut Activation<'_, 'gc>) {
        let write = Gc::write(activation.context.gc(), self.0);
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
                activation.context.stop_sound(sound_instance);
            }
        }

        let owner = self.owner(activation);
        let sound_instance = activation.context.start_sound(
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
            None,
            owner,
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

    pub fn load_sound(
        self,
        activation: &mut Activation<'_, 'gc>,
        sound_object: Object<'gc>,
        sound: SoundHandle,
    ) {
        let new_data = SoundState::Loaded { sound };
        let write = Gc::write(activation.gc(), self.0);
        let old_data = unlock!(write, SoundData, state).replace(new_data);

        if let SoundState::Loading { queued_plays } = old_data {
            for play in queued_plays {
                self.play(play, activation);
            }
        }

        if !sound_object.has_property(activation, istr!("position"))
            || !sound_object.has_property(activation, istr!("duration"))
        {
            let fn_proto = activation.prototypes().function;
            let getter_position =
                FunctionObject::native(position).build(activation.strings(), fn_proto, None);
            let getter_duration =
                FunctionObject::native(duration).build(activation.strings(), fn_proto, None);

            sound_object.add_property(
                activation.gc(),
                istr!("position"),
                getter_position,
                None,
                Attribute::DONT_ENUM | Attribute::DONT_DELETE,
            );
            sound_object.add_property(
                activation.gc(),
                istr!("duration"),
                getter_duration,
                None,
                Attribute::DONT_ENUM | Attribute::DONT_DELETE,
            );
        }
    }

    pub fn load_id3(
        self,
        activation: &mut Activation<'_, 'gc>,
        sound_object: Object<'gc>,
        bytes: &[u8],
    ) -> Result<(), Error<'gc>> {
        let tag = Tag::read_from2(Cursor::new(bytes));
        let Ok(ref tag) = tag else {
            // no ID3, or malformed
            return Ok(());
        };

        let id3 = Object::new_without_proto(activation.gc());

        for frame in tag.frames() {
            let id_2_0 = frame.id();
            let id_2_0_avm = AvmString::new_utf8(activation.gc(), id_2_0);
            let id_1_0 = match id_2_0 {
                "COMM" => Some(istr!("comment")),
                "TALB" => Some(istr!("album")),
                "TCON" => Some(istr!("genre")),
                "TIT2" => Some(istr!("songname")),
                "TPE1" => Some(istr!("artist")),
                "TRCK" => Some(istr!("track")),
                "TYER" => Some(istr!("year")),
                _ => None,
            };

            let (value_1_0, value_2_0): (Value<'gc>, Value<'gc>) = match frame.content() {
                id3::Content::Text(text) => {
                    let value = AvmString::new_utf8(activation.gc(), text).into();
                    (value, value)
                }
                id3::Content::Comment(comment) => {
                    let value = AvmString::new_utf8(activation.gc(), &comment.text).into();

                    let comment_array =
                        if let Value::Object(comment_array) = id3.get(id_2_0_avm, activation)? {
                            comment_array
                        } else {
                            ArrayBuilder::empty(activation)
                        };

                    let len = comment_array.length(activation)?;
                    comment_array.set_element(activation, len, value)?;
                    comment_array.set_length(activation, len + 1)?;

                    // ID3 1.0 value is always the last comment.
                    // ID3 2.0 value aggregates all comments into an array.
                    (value, comment_array.into())
                }
                _ => continue,
            };

            if let Some(id_1_0) = id_1_0 {
                id3.set(id_1_0, value_1_0, activation)?;
            }

            // TODO: This probably should be set for FP 7+ only
            id3.set(id_2_0_avm, value_2_0, activation)?;
        }

        if !sound_object.has_property(activation, istr!("id3")) {
            sound_object.set(istr!("id3"), id3, activation)?;
            sound_object.set_attributes(
                activation.gc(),
                Some(istr!("id3")),
                Attribute::DONT_ENUM | Attribute::DONT_DELETE | Attribute::READ_ONLY,
                Attribute::empty(),
            );
        }

        let _ = sound_object.call_method(
            istr!("onID3"),
            // Flash always passes true as the first argument, even if docs say
            // the function does not accept anything. Can it be false?
            &[true.into()],
            activation,
            ExecutionReason::Special,
        );
        Ok(())
    }
}

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    // Note: id3 is not a built-in property. See [`Sound::load_id3`].
    // Note: duration is defined later. See [`Sound::load_sound`].
    // Note: position is defined later. See [`Sound::load_sound`].
    "getPan" => method(get_pan; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getTransform" => method(get_transform; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getVolume" => method(get_volume; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setPan" => method(set_pan; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setTransform" => method(set_transform; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setVolume" => method(set_volume; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "stop" => method(stop; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "attachSound" => method(attach_sound; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "start" => method(start; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getDuration" => method(duration; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_6);
    "setDuration" => method(set_duration; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_6);
    "getPosition" => method(position; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_6);
    "setPosition" => method(set_position; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_6);
    "loadSound" => method(load_sound; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_6);
    "getBytesLoaded" => method(get_bytes_loaded; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_6);
    "getBytesTotal" => method(get_bytes_total; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_6);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.native_class(
        constructor,
        None,
        super_proto,
        PropertyOrder::PrototypeFirst,
    );
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    class
}

/// Implements `Sound`
fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // 1st parameter is the display object that "owns" all sounds started by this object.
    // If `null` or `undefined` is provided, then the sound will be set to always
    // control the global sound. Otherwise, it gets coerced to a string path and will
    // try to be resolved every time a method needs it.
    // `Sound.setTransform`, `Sound.stop`, etc. will affect all sounds owned by the target clip.

    let target = args
        .get(0)
        // Null or Undefined means that we will use global sound
        .filter(|v| !matches!(v, Value::Null | Value::Undefined))
        .map(|value| value.coerce_to_string(activation))
        .transpose()?;

    let sound = Sound::empty(activation.gc(), target);
    this.set_native(activation.gc(), NativeObject::Sound(sound));

    Ok(this.into())
}

fn attach_sound<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::Sound(sound) = this.native() {
        let name = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_string(activation)?;

        let owner = if sound.use_global_sound() {
            activation.base_clip().avm1_root()
        } else if let Some(owner) = sound.owner(activation) {
            owner
        } else {
            return Ok(Value::Undefined);
        };

        if let Some((_, Character::Sound(sound_handle))) = owner
            .library()
            .unwrap()
            .borrow()
            .character_by_export_name(name)
        {
            sound.load_sound(activation, this, sound_handle);
            sound.set_is_streaming(false);
            sound.set_duration(
                activation
                    .context
                    .audio
                    .get_sound_duration(sound_handle)
                    .map(|d| d.as_millis().round() as u32),
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
        if sound.has_valid_owner(activation) {
            return Ok(sound.duration().map_or(Value::Undefined, |d| d.into()));
        }
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
    avm1_stub!(activation, "Sound", "getBytesLoaded");
    Ok(1.into())
}

fn get_bytes_total<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Sound", "getBytesTotal");
    Ok(1.into())
}

fn get_pan<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::Sound(sound) = this.native()
        && let Some(transform) = sound.sound_transform(activation)
    {
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
    if let NativeObject::Sound(sound) = this.native()
        && let Some(transform) = sound.sound_transform(activation)
    {
        let obj = Object::new(
            &activation.context.strings,
            Some(activation.prototypes().object),
        );
        // Surprisingly `lr` means "right-to-left" and `rl` means "left-to-right".
        obj.set(istr!("ll"), transform.left_to_left, activation)?;
        obj.set(istr!("lr"), transform.right_to_left, activation)?;
        obj.set(istr!("rr"), transform.right_to_right, activation)?;
        obj.set(istr!("rl"), transform.left_to_right, activation)?;
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
    if let NativeObject::Sound(sound) = this.native()
        && let Some(transform) = sound.sound_transform(activation)
    {
        Ok(transform.volume.into())
    } else {
        Ok(Value::Undefined)
    }
}

fn load_sound<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::Sound(sound) = this.native()
        && sound.has_valid_owner(activation)
        && let Some(url) = args.get(0)
    {
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
        let future = crate::loader::load_sound_avm1(
            activation.context,
            this,
            Request::get(url.to_utf8_lossy().into_owned()),
            is_streaming,
        );
        activation.context.navigator.spawn_future(future);
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
        if sound.sound().is_some() && sound.has_valid_owner(activation) {
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
    if let NativeObject::Sound(sound) = this.native() {
        let pan = args
            .get(0)
            .unwrap_or(&0.into())
            .coerce_to_f64(activation)?
            .clamp_to_i32();
        if let Some(owner) = sound.owner(activation) {
            let transform = owner.base().sound_transform();
            owner.set_sound_transform(activation.context, transform.with_pan(pan));
        } else if sound.use_global_sound() {
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
    if let NativeObject::Sound(sound) = this.native() {
        let obj = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object_or_bare(activation)?;

        let owner = sound.owner(activation);

        if owner.is_none() && !sound.use_global_sound() {
            return Ok(Value::Undefined);
        }

        let mut transform = owner.map_or_else(
            || *activation.context.global_sound_transform(),
            |owner| owner.base().sound_transform(),
        );

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

        if let Some(owner) = owner {
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
    if let NativeObject::Sound(sound) = this.native() {
        let volume = args
            .get(0)
            .unwrap_or(&0.into())
            .coerce_to_f64(activation)?
            .clamp_to_i32();
        if let Some(owner) = sound.owner(activation) {
            let transform = SoundTransform {
                volume,
                ..owner.base().sound_transform()
            };
            owner.set_sound_transform(activation.context, transform);
        } else if sound.use_global_sound() {
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
    if let NativeObject::Sound(sound) = this.native() {
        if sound.has_valid_owner(activation) {
            let start_offset = args.get(0).unwrap_or(&0.into()).coerce_to_f64(activation)?;
            let loops = args.get(1).unwrap_or(&1.into()).coerce_to_f64(activation)?;

            // TODO: Handle loops > u16::MAX.
            let loops = (loops as u16).max(1);
            let play = QueuedPlay {
                sound_object: this,
                start_offset,
                loops,
            };
            sound.play(play, activation);
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
            let owner = if sound.use_global_sound() {
                activation.base_clip().avm1_root()
            } else if let Some(owner) = sound.owner(activation) {
                owner
            } else {
                return Ok(Value::Undefined);
            };

            if let Some((_, Character::Sound(sound))) = owner
                .library()
                .unwrap()
                .borrow()
                .character_by_export_name(name)
            {
                // TODO: This isn't entirely correct. We should only
                // stop sounds with this name on this particular sound object;
                // right now we're stopping *all* sound objects playing this sound.
                activation.context.stop_sounds_with_handle(sound);
            } else {
                avm_warn!(activation, "Sound.stop: Sound '{}' not found", name);
            }
        } else if let Some(owner) = sound.owner(activation) {
            // Usage 2: If there is no name and we have an owner,
            // then stop all sound running within a given clip.
            activation.context.stop_sounds_with_display_object(owner);
            sound.set_sound_instance(None);
        } else if sound.use_global_sound() {
            // Usage 3: If there is no name and we are linked to global sound,
            // this call acts like `stopAllSounds()`.
            activation.context.stop_all_sounds();
        }
    } else {
        avm_warn!(activation, "Sound.stop: this is not a Sound");
    }

    Ok(Value::Undefined)
}

fn set_position<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Sound", "setPosition");
    Ok(Value::Undefined)
}
