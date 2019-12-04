//! AVM1 object type to represent objects on the stage.

use crate::avm1::function::Executable;
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ObjectPtr, ScriptObject, TDisplayObject, TObject, Value};
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, MovieClip};
use enumset::EnumSet;
use gc_arena::{Collect, GcCell, MutationContext};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// The type string for MovieClip objects.
pub const TYPE_OF_MOVIE_CLIP: &str = "movieclip";

/// A ScriptObject that is inherently tied to a display node.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct StageObject<'gc> {
    /// The underlying script object.
    ///
    /// This is used to handle "expando properties" on AVM1 display nodes, as
    /// well as the underlying prototype chain.
    base: ScriptObject<'gc>,

    /// The display node this stage object
    display_object: DisplayObject<'gc>,
}

impl<'gc> StageObject<'gc> {
    /// Create a stage object for a given display node.
    pub fn for_display_object(
        gc_context: MutationContext<'gc, '_>,
        display_object: DisplayObject<'gc>,
        proto: Option<Object<'gc>>,
    ) -> Self {
        let mut base = ScriptObject::object(gc_context, proto);

        //TODO: Do other display node objects have different typestrings?
        base.set_type_of(gc_context, TYPE_OF_MOVIE_CLIP);

        Self {
            base,
            display_object,
        }
    }
}

impl fmt::Debug for StageObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StageObject")
            .field("base", &self.base)
            .field("display_object", &self.display_object)
            .finish()
    }
}

impl<'gc> TObject<'gc> for StageObject<'gc> {
    fn get(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        let props = avm.display_properties;
        // Property search order for DisplayObjects:
        if self.has_own_property(name) {
            // 1) Actual properties on the underlying object
            self.get_local(name, avm, context, (*self).into())
        } else if let Some(property) = props.read().get_by_name(&name) {
            // 2) Display object properties such as _x, _y
            let val = property.get(avm, context, self.display_object)?;
            Ok(val.into())
        } else if let Some(child) = self.display_object.get_child_by_name(name) {
            // 3) Child display objects with the given instance name
            Ok(child.object().into())
        } else {
            // 4) Prototype
            crate::avm1::object::search_prototype(self.proto(), name, avm, context, (*self).into())
        }
        // 4) TODO: __resolve?
    }

    fn get_local(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<ReturnValue<'gc>, Error> {
        self.base.get_local(name, avm, context, this)
    }

    fn set(
        &self,
        name: &str,
        value: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let props = avm.display_properties;
        if self.base.has_own_property(name) {
            // 1) Actual proeprties on the underlying object
            self.base.set(name, value, avm, context)
        } else if let Some(property) = props.read().get_by_name(&name) {
            // 2) Display object properties such as _x, _y
            property.set(avm, context, self.display_object, value)?;
            Ok(())
        } else {
            // 3) TODO: Prototype
            self.base.set(name, value, avm, context)
        }
    }

    fn call(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<ReturnValue<'gc>, Error> {
        self.base.call(avm, context, this, args)
    }

    #[allow(clippy::new_ret_no_self)]
    fn new(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        //TODO: Create a StageObject of some kind
        self.base.new(avm, context, this, args)
    }
    fn delete(&self, gc_context: MutationContext<'gc, '_>, name: &str) -> bool {
        self.base.delete(gc_context, name)
    }
    fn proto(&self) -> Option<Object<'gc>> {
        self.base.proto()
    }
    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base.define_value(gc_context, name, value, attributes)
    }

    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Executable<'gc>,
        set: Option<Executable<'gc>>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base
            .add_property(gc_context, name, get, set, attributes)
    }

    fn has_property(&self, name: &str) -> bool {
        if self.base.has_property(name) {
            return true;
        }

        if self.display_object.get_child_by_name(name).is_some() {
            return true;
        }

        false
    }

    fn has_own_property(&self, name: &str) -> bool {
        // Note that `hasOwnProperty` does NOT return true for child display objects.
        self.base.has_own_property(name)
    }

    fn is_property_enumerable(&self, name: &str) -> bool {
        self.base.is_property_enumerable(name)
    }

    fn is_property_overwritable(&self, name: &str) -> bool {
        self.base.is_property_overwritable(name)
    }

    fn get_keys(&self) -> HashSet<String> {
        // Keys from the underlying object are listed first, followed by
        // child display objects in order from highest depth to lowest depth.
        // TODO: It's possible to have multiple instances with the same name,
        // and that name will be returned multiple times in the key list for a `for..in` loop.
        let mut keys = self.base.get_keys();
        for child in self.display_object.children() {
            keys.insert(child.name().to_string());
        }
        keys
    }

    fn as_string(&self) -> String {
        self.display_object.path()
    }

    fn type_of(&self) -> &'static str {
        self.base.type_of()
    }
    fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
        Some(self.base)
    }

    fn as_display_object(&self) -> Option<DisplayObject<'gc>> {
        Some(self.display_object)
    }
    fn as_executable(&self) -> Option<Executable<'gc>> {
        None
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.base.as_ptr() as *const ObjectPtr
    }
}

/// Properties shared by display objects in AVM1, such as _x and _y.
/// These are only accessible for movie clips, buttons, and text fields (any others?)
/// These exist outside the global or prototype machinery. Instead, they are
/// "special" properties stored in a separate map that display objects look at in addition
/// to normal property lookup.
/// The map of property names to display object getts/setters.
#[derive(Copy, Clone)]
pub struct DisplayProperty<'gc> {
    get: DisplayGetter<'gc>,
    set: Option<DisplaySetter<'gc>>,
}

pub type DisplayGetter<'gc> = fn(
    &mut Avm1<'gc>,
    &mut UpdateContext<'_, 'gc, '_>,
    DisplayObject<'gc>,
) -> Result<Value<'gc>, Error>;

pub type DisplaySetter<'gc> = fn(
    &mut Avm1<'gc>,
    &mut UpdateContext<'_, 'gc, '_>,
    DisplayObject<'gc>,
    Value<'gc>,
) -> Result<(), Error>;

impl<'gc> DisplayProperty<'gc> {
    pub fn get(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: DisplayObject<'gc>,
    ) -> Result<Value<'gc>, Error> {
        (self.get)(avm, context, this)
    }

    pub fn set(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: DisplayObject<'gc>,
        value: Value<'gc>,
    ) -> Result<(), Error> {
        self.set
            .map(|f| f(avm, context, this, value))
            .unwrap_or(Ok(()))
    }
}

unsafe impl<'gc> Collect for DisplayProperty<'gc> {
    fn needs_trace() -> bool {
        false
    }
}

/// The map from key/index to function pointers for special display object properties.
#[derive(Collect)]
#[collect(no_drop)]
pub struct DisplayPropertyMap<'gc> {
    property_by_name: HashMap<String, DisplayProperty<'gc>>,
    property_by_index: Vec<DisplayProperty<'gc>>,
}

impl<'gc> DisplayPropertyMap<'gc> {
    /// Creates the display property map.
    pub fn new(gc_context: MutationContext<'gc, '_>) -> GcCell<'gc, DisplayPropertyMap<'gc>> {
        let mut property_map = DisplayPropertyMap {
            property_by_name: HashMap::with_capacity(21),
            property_by_index: Vec::with_capacity(21),
        };

        // Order is important:
        // should match the SWF specs for GetProperty/SetProperty.
        property_map.add_property("_x", x, Some(set_x));
        property_map.add_property("_y", y, Some(set_y));
        property_map.add_property("_xscale", x_scale, Some(set_x_scale));
        property_map.add_property("_yscale", y_scale, Some(set_y_scale));
        property_map.add_property("_currentframe", current_frame, None);
        property_map.add_property("_totalframes", total_frames, None);
        property_map.add_property("_alpha", alpha, Some(set_alpha));
        property_map.add_property("_visible", visible, Some(set_visible));
        property_map.add_property("_width", width, Some(set_width));
        property_map.add_property("_height", height, Some(set_height));
        property_map.add_property("_rotation", rotation, Some(set_rotation));
        property_map.add_property("_target", target, None);
        property_map.add_property("_framesloaded", frames_loaded, None);
        property_map.add_property("_name", name, None);
        property_map.add_property("_droptarget", drop_target, None);
        property_map.add_property("_url", url, None);
        property_map.add_property("_highquality", high_quality, Some(set_high_quality));
        property_map.add_property("_focusrect", focus_rect, Some(set_focus_rect));
        property_map.add_property("_soundbuftime", sound_buf_time, Some(set_sound_buf_time));
        property_map.add_property("_quality", quality, Some(set_quality));
        property_map.add_property("_xmouse", x_mouse, None);
        property_map.add_property("_ymouse", y_mouse, None);

        GcCell::allocate(gc_context, property_map)
    }

    /// Gets a property slot by name.
    /// Used by `GetMember`, `GetVariable`, `SetMember`, and `SetVariable`.
    pub fn get_by_name(&self, name: &str) -> Option<&DisplayProperty<'gc>> {
        // Display object properties are case insensitive, regardless of SWF version!?
        // TODO: Another string alloc; optimize this eventually.
        self.property_by_name.get(&name.to_ascii_lowercase())
    }

    /// Gets a property slot by SWF4 index.
    /// The order is defined by the SWF specs.
    /// Used by `GetProperty`/`SetProperty`.
    /// SWF19 pp. 85-86
    pub fn get_by_index(&self, index: usize) -> Option<&DisplayProperty<'gc>> {
        self.property_by_index.get(index)
    }

    fn add_property(
        &mut self,
        name: &str,
        get: DisplayGetter<'gc>,
        set: Option<DisplaySetter<'gc>>,
    ) {
        let prop = DisplayProperty { get, set };
        self.property_by_name.insert(name.to_string(), prop.clone());
        self.property_by_index.push(prop);
    }
}

fn x<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    Ok(this.x().into())
}

fn set_x<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    mut this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error> {
    let val = val.as_number(avm, context)?;
    this.set_x(context.gc_context, val);
    Ok(())
}

fn y<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    Ok(this.y().into())
}

fn set_y<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    mut this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error> {
    let val = val.as_number(avm, context)?;
    this.set_y(context.gc_context, val);
    Ok(())
}

fn x_scale<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    mut this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    let val = this.scale_x(context.gc_context) * 100.0;
    Ok(val.into())
}

fn set_x_scale<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    mut this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error> {
    let val = val.as_number(avm, context)? / 100.0;
    this.set_scale_x(context.gc_context, val);
    Ok(())
}

fn y_scale<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    mut this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    let scale_y = this.scale_y(context.gc_context) * 100.0;
    Ok(scale_y.into())
}

fn set_y_scale<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    mut this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error> {
    let val = val.as_number(avm, context)? / 100.0;
    this.set_scale_y(context.gc_context, val);
    Ok(())
}

fn current_frame<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    log::warn!("Unimplemented property _currentframe");
    Ok(this
        .as_movie_clip()
        .map(MovieClip::current_frame)
        .map(Value::from)
        .unwrap_or(Value::Undefined))
}

fn total_frames<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    log::warn!("Unimplemented property _totalframes");
    Ok(this
        .as_movie_clip()
        .map(MovieClip::total_frames)
        .map(Value::from)
        .unwrap_or(Value::Undefined))
}

fn alpha<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    let val = this.alpha() * 100.0;
    Ok(val.into())
}

fn set_alpha<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error> {
    let val = val.as_number(avm, context)? / 100.0;
    this.set_alpha(context.gc_context, val);
    Ok(())
}

fn visible<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    let val = this.visible();
    Ok(val.into())
}

fn set_visible<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    mut this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error> {
    let val = val.as_bool(avm.current_swf_version());
    this.set_visible(context.gc_context, val);
    Ok(())
}

fn width<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    Ok(this.width().into())
}

fn set_width<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    mut this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error> {
    let val = val.as_number(avm, context)?;
    this.set_width(context.gc_context, val);
    Ok(())
}

fn height<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    Ok(this.height().into())
}

fn set_height<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    mut this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error> {
    let val = val.as_number(avm, context)?;
    this.set_height(context.gc_context, val);
    Ok(())
}

fn rotation<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    mut this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    Ok(this.rotation(context.gc_context).to_degrees().into())
}

fn set_rotation<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    mut this: DisplayObject<'gc>,
    degrees: Value<'gc>,
) -> Result<(), Error> {
    let mut degrees = degrees.as_number(avm, context)?;
    // Normalize into the range of [-180, 180].
    degrees %= 360.0;
    if degrees < -180.0 {
        degrees += 360.0
    } else if degrees > 180.0 {
        degrees -= 360.0
    }
    this.set_rotation(context.gc_context, degrees.to_radians());
    Ok(())
}

fn target<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    log::warn!("Unimplemented property _target");
    Ok("".into())
}

fn frames_loaded<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    Ok(this
        .as_movie_clip()
        .map(MovieClip::frames_loaded)
        .map(Value::from)
        .unwrap_or(Value::Undefined))
}

fn name<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    Ok((*this.name()).into())
}

fn drop_target<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    log::warn!("Unimplemented property _droptarget");
    Ok("".into())
}

fn url<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    log::warn!("Unimplemented property _url");
    Ok("".into())
}

fn high_quality<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    log::warn!("Unimplemented property _highquality");
    Ok(1.into())
}

fn set_high_quality<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
    _val: Value<'gc>,
) -> Result<(), Error> {
    log::warn!("Unimplemented property _highquality");
    Ok(())
}

fn focus_rect<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    log::warn!("Unimplemented property _focusrect");
    Ok(Value::Null)
}

fn set_focus_rect<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
    _val: Value<'gc>,
) -> Result<(), Error> {
    log::warn!("Unimplemented property _focusrect");
    Ok(())
}

fn sound_buf_time<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    log::warn!("Unimplemented property _soundbuftime");
    Ok(5.into())
}

fn set_sound_buf_time<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
    _val: Value<'gc>,
) -> Result<(), Error> {
    log::warn!("Unimplemented property _soundbuftime");
    Ok(())
}

fn quality<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    log::warn!("Unimplemented property _quality");
    Ok("HIGH".into())
}

fn set_quality<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
    _val: Value<'gc>,
) -> Result<(), Error> {
    log::warn!("Unimplemented property _quality");
    Ok(())
}

fn x_mouse<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    log::warn!("Unimplemented property _xmouse");
    Ok(Value::Undefined)
}

fn y_mouse<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error> {
    log::warn!("Unimplemented property _ymouse");
    Ok(Value::Undefined)
}
