//! AVM1 object type to represent objects on the stage.

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::Executable;
use crate::avm1::object::search_prototype;
use crate::avm1::property::Attribute;
use crate::avm1::{AvmString, Object, ObjectPtr, ScriptObject, TDisplayObject, TObject, Value};
use crate::avm_warn;
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, EditText, MovieClip};
use crate::property_map::PropertyMap;
use enumset::EnumSet;
use gc_arena::{Collect, GcCell, MutationContext};
use std::borrow::Cow;
use std::fmt;

/// The type string for MovieClip objects.
pub const TYPE_OF_MOVIE_CLIP: &str = "movieclip";

/// A ScriptObject that is inherently tied to a display node.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct StageObject<'gc>(GcCell<'gc, StageObjectData<'gc>>);

#[derive(Collect)]
#[collect(no_drop)]
pub struct StageObjectData<'gc> {
    /// The underlying script object.
    ///
    /// This is used to handle "expando properties" on AVM1 display nodes, as
    /// well as the underlying prototype chain.
    base: ScriptObject<'gc>,

    /// The display node this stage object
    display_object: DisplayObject<'gc>,

    text_field_bindings: Vec<TextFieldBinding<'gc>>,
}

impl<'gc> StageObject<'gc> {
    /// Create a stage object for a given display node.
    pub fn for_display_object(
        gc_context: MutationContext<'gc, '_>,
        display_object: DisplayObject<'gc>,
        proto: Option<Object<'gc>>,
    ) -> Self {
        let mut base = ScriptObject::object(gc_context, proto);

        // Movieclips have a special typeof "movieclip", while others are the default "object".
        if display_object.as_movie_clip().is_some() {
            base.set_type_of(gc_context, TYPE_OF_MOVIE_CLIP);
        }

        Self(GcCell::allocate(
            gc_context,
            StageObjectData {
                base,
                display_object,
                text_field_bindings: Vec::new(),
            },
        ))
    }

    /// Registers a text field variable binding for this stage object.
    /// Whenever a property with the given name is changed, we should change the text in the text field.
    pub fn register_text_field_binding(
        self,
        gc_context: MutationContext<'gc, '_>,
        text_field: EditText<'gc>,
        variable_name: &str,
    ) {
        self.0
            .write(gc_context)
            .text_field_bindings
            .push(TextFieldBinding {
                text_field,
                variable_name: variable_name.to_string(),
            })
    }

    /// Removes a text field binding for the given text field.
    /// Does not place the text field on the unbound list.
    /// Caller is responsible for placing the text field on the unbound list, if necessary.
    pub fn clear_text_field_binding(
        self,
        gc_context: MutationContext<'gc, '_>,
        text_field: EditText<'gc>,
    ) {
        self.0
            .write(gc_context)
            .text_field_bindings
            .retain(|binding| !DisplayObject::ptr_eq(text_field.into(), binding.text_field.into()));
    }

    /// Clears all text field bindings from this stage object, and places the textfields on the unbound list.
    /// This is called when the object is removed from the stage.
    pub fn unregister_text_field_bindings(self, context: &mut UpdateContext<'_, 'gc, '_>) {
        for binding in self
            .0
            .write(context.gc_context)
            .text_field_bindings
            .drain(..)
        {
            binding.text_field.clear_bound_stage_object(context);
            context.unbound_text_fields.push(binding.text_field);
        }
    }
}

/// A binding from a property of this StageObject to an EditText text field.
#[derive(Collect)]
#[collect(no_drop)]
struct TextFieldBinding<'gc> {
    text_field: EditText<'gc>,
    variable_name: String,
}

impl fmt::Debug for StageObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let o = self.0.read();
        f.debug_struct("StageObject")
            .field("base", &o.base)
            .field("display_object", &o.display_object)
            .finish()
    }
}

impl<'gc> TObject<'gc> for StageObject<'gc> {
    fn get(
        &self,
        name: &str,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let obj = self.0.read();
        let props = activation.context.avm1.display_properties;
        let case_sensitive = activation.is_case_sensitive();
        // Property search order for DisplayObjects:
        if self.has_own_property(activation, name) {
            // 1) Actual properties on the underlying object
            self.get_local(name, activation, (*self).into())
        } else if let Some(property) = props.read().get_by_name(&name) {
            // 2) Display object properties such as _x, _y
            let val = property.get(activation, obj.display_object)?;
            Ok(val)
        } else if let Some(child) = obj.display_object.get_child_by_name(name, case_sensitive) {
            // 3) Child display objects with the given instance name
            Ok(child.object())
        } else if let Some(level) =
            obj.display_object
                .get_level_by_path(name, &mut activation.context, case_sensitive)
        {
            // 4) _levelN
            Ok(level.object())
        } else {
            // 5) Prototype
            Ok(search_prototype(self.proto(), name, activation, (*self).into())?.0)
        }
        // 6) TODO: __resolve?
    }

    fn get_local(
        &self,
        name: &str,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        self.0.read().base.get_local(name, activation, this)
    }

    fn set(
        &self,
        name: &str,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error<'gc>> {
        let obj = self.0.read();
        let props = activation.context.avm1.display_properties;

        // Check if a text field is bound to this property and update the text if so.
        for binding in obj
            .text_field_bindings
            .iter()
            .filter(|binding| binding.variable_name == name)
        {
            let _ = binding.text_field.set_html_text(
                value.coerce_to_string(activation)?.to_string(),
                &mut activation.context,
            );
        }

        if obj.base.has_own_property(activation, name) {
            // 1) Actual proeprties on the underlying object
            obj.base.internal_set(
                name,
                value,
                activation,
                (*self).into(),
                Some((*self).into()),
            )
        } else if let Some(property) = props.read().get_by_name(&name) {
            // 2) Display object properties such as _x, _y
            property.set(activation, obj.display_object, value)?;
            Ok(())
        } else {
            // 3) TODO: Prototype
            obj.base.internal_set(
                name,
                value,
                activation,
                (*self).into(),
                Some((*self).into()),
            )
        }
    }
    fn call(
        &self,
        name: &str,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
        base_proto: Option<Object<'gc>>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        self.0
            .read()
            .base
            .call(name, activation, this, base_proto, args)
    }

    fn call_setter(
        &self,
        name: &str,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Object<'gc>> {
        self.0.read().base.call_setter(name, value, activation)
    }

    #[allow(clippy::new_ret_no_self)]
    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        //TODO: Create a StageObject of some kind
        self.0.read().base.create_bare_object(activation, this)
    }

    fn delete(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.0.read().base.delete(activation, name)
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.0.read().base.proto()
    }

    fn set_proto(&self, gc_context: MutationContext<'gc, '_>, prototype: Option<Object<'gc>>) {
        self.0.read().base.set_proto(gc_context, prototype);
    }

    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    ) {
        self.0
            .read()
            .base
            .define_value(gc_context, name, value, attributes)
    }

    fn set_attributes(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: Option<&str>,
        set_attributes: EnumSet<Attribute>,
        clear_attributes: EnumSet<Attribute>,
    ) {
        self.0.write(gc_context).base.set_attributes(
            gc_context,
            name,
            set_attributes,
            clear_attributes,
        )
    }

    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Object<'gc>,
        set: Option<Object<'gc>>,
        attributes: EnumSet<Attribute>,
    ) {
        self.0
            .read()
            .base
            .add_property(gc_context, name, get, set, attributes)
    }

    fn add_property_with_case(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Object<'gc>,
        set: Option<Object<'gc>>,
        attributes: EnumSet<Attribute>,
    ) {
        self.0
            .read()
            .base
            .add_property_with_case(activation, gc_context, name, get, set, attributes)
    }

    fn set_watcher(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        gc_context: MutationContext<'gc, '_>,
        name: Cow<str>,
        callback: Object<'gc>,
        user_data: Value<'gc>,
    ) {
        self.0
            .read()
            .base
            .set_watcher(activation, gc_context, name, callback, user_data);
    }

    fn remove_watcher(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        gc_context: MutationContext<'gc, '_>,
        name: Cow<str>,
    ) -> bool {
        self.0
            .read()
            .base
            .remove_watcher(activation, gc_context, name)
    }

    fn has_property(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        let obj = self.0.read();
        if obj.base.has_property(activation, name) {
            return true;
        }

        if activation
            .context
            .avm1
            .display_properties
            .read()
            .get_by_name(&name)
            .is_some()
        {
            return true;
        }

        let case_sensitive = activation.is_case_sensitive();
        if obj
            .display_object
            .get_child_by_name(name, case_sensitive)
            .is_some()
        {
            return true;
        }

        if obj
            .display_object
            .get_level_by_path(name, &mut activation.context, case_sensitive)
            .is_some()
        {
            return true;
        }

        false
    }

    fn has_own_property(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        // Note that `hasOwnProperty` does NOT return true for child display objects.
        self.0.read().base.has_own_property(activation, name)
    }

    fn has_own_virtual(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.0.read().base.has_own_virtual(activation, name)
    }

    fn is_property_enumerable(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.0.read().base.is_property_enumerable(activation, name)
    }

    fn get_keys(&self, activation: &mut Activation<'_, 'gc, '_>) -> Vec<String> {
        // Keys from the underlying object are listed first, followed by
        // child display objects in order from highest depth to lowest depth.
        let obj = self.0.read();
        let mut keys = obj.base.get_keys(activation);
        keys.extend(
            obj.display_object
                .children()
                .map(|child| child.name().to_string()),
        );
        keys
    }

    fn length(&self) -> usize {
        self.0.read().base.length()
    }

    fn set_length(&self, gc_context: MutationContext<'gc, '_>, new_length: usize) {
        self.0.read().base.set_length(gc_context, new_length)
    }

    fn array(&self) -> Vec<Value<'gc>> {
        self.0.read().base.array()
    }

    fn array_element(&self, index: usize) -> Value<'gc> {
        self.0.read().base.array_element(index)
    }

    fn set_array_element(
        &self,
        index: usize,
        value: Value<'gc>,
        gc_context: MutationContext<'gc, '_>,
    ) -> usize {
        self.0
            .read()
            .base
            .set_array_element(index, value, gc_context)
    }

    fn delete_array_element(&self, index: usize, gc_context: MutationContext<'gc, '_>) {
        self.0.read().base.delete_array_element(index, gc_context)
    }

    fn interfaces(&self) -> Vec<Object<'gc>> {
        self.0.read().base.interfaces()
    }

    fn set_interfaces(&self, context: MutationContext<'gc, '_>, iface_list: Vec<Object<'gc>>) {
        self.0
            .write(context)
            .base
            .set_interfaces(context, iface_list)
    }

    fn as_string(&self) -> Cow<str> {
        Cow::Owned(self.0.read().display_object.path())
    }

    fn type_of(&self) -> &'static str {
        self.0.read().base.type_of()
    }
    fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
        Some(self.0.read().base)
    }

    /// Get the underlying stage object, if it exists.
    fn as_stage_object(&self) -> Option<StageObject<'gc>> {
        Some(*self)
    }

    fn as_display_object(&self) -> Option<DisplayObject<'gc>> {
        Some(self.0.read().display_object)
    }
    fn as_executable(&self) -> Option<Executable<'gc>> {
        None
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.read().base.as_ptr() as *const ObjectPtr
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

pub type DisplayGetter<'gc> =
    fn(&mut Activation<'_, 'gc, '_>, DisplayObject<'gc>) -> Result<Value<'gc>, Error<'gc>>;

pub type DisplaySetter<'gc> =
    fn(&mut Activation<'_, 'gc, '_>, DisplayObject<'gc>, Value<'gc>) -> Result<(), Error<'gc>>;

impl<'gc> DisplayProperty<'gc> {
    pub fn get(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: DisplayObject<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        (self.get)(activation, this)
    }

    pub fn set(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: DisplayObject<'gc>,
        value: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        self.set
            .map(|f| f(activation, this, value))
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
pub struct DisplayPropertyMap<'gc>(PropertyMap<DisplayProperty<'gc>>);

impl<'gc> DisplayPropertyMap<'gc> {
    /// Creates the display property map.
    pub fn new(gc_context: MutationContext<'gc, '_>) -> GcCell<'gc, DisplayPropertyMap<'gc>> {
        let mut property_map = DisplayPropertyMap(PropertyMap::new());

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
        property_map.add_property("_name", name, Some(set_name));
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
        self.0.get(&name, false)
    }

    /// Gets a property slot by SWF4 index.
    /// The order is defined by the SWF specs.
    /// Used by `GetProperty`/`SetProperty`.
    /// SWF19 pp. 85-86
    pub fn get_by_index(&self, index: usize) -> Option<&DisplayProperty<'gc>> {
        self.0.get_index(index)
    }

    fn add_property(
        &mut self,
        name: &str,
        get: DisplayGetter<'gc>,
        set: Option<DisplaySetter<'gc>>,
    ) {
        let prop = DisplayProperty { get, set };
        self.0.insert(name, prop, false);
    }
}

fn x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.x().into())
}

fn set_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(val) = property_coerce_to_number(activation, val)? {
        this.set_x(activation.context.gc_context, val);
    }
    Ok(())
}

fn y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.y().into())
}

fn set_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(val) = property_coerce_to_number(activation, val)? {
        this.set_y(activation.context.gc_context, val);
    }
    Ok(())
}

fn x_scale<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let val = this.scale_x(activation.context.gc_context) * 100.0;
    Ok(val.into())
}

fn set_x_scale<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(val) = property_coerce_to_number(activation, val)? {
        this.set_scale_x(activation.context.gc_context, val / 100.0);
    }
    Ok(())
}

fn y_scale<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let scale_y = this.scale_y(activation.context.gc_context) * 100.0;
    Ok(scale_y.into())
}

fn set_y_scale<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(val) = property_coerce_to_number(activation, val)? {
        this.set_scale_y(activation.context.gc_context, val / 100.0);
    }
    Ok(())
}

fn current_frame<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_movie_clip()
        .map(MovieClip::current_frame)
        .map(Value::from)
        .unwrap_or(Value::Undefined))
}

fn total_frames<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_movie_clip()
        .map(MovieClip::total_frames)
        .map(Value::from)
        .unwrap_or(Value::Undefined))
}

fn alpha<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let val = this.alpha() * 100.0;
    Ok(val.into())
}

fn set_alpha<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(val) = property_coerce_to_number(activation, val)? {
        this.set_alpha(activation.context.gc_context, val / 100.0);
    }
    Ok(())
}

fn visible<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let val = this.visible();
    Ok(val.into())
}

fn set_visible<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    // Because this property dates to the era of Flash 4, this is actually coerced to an integer.
    // `_visible = "false";` coerces to NaN and has no effect.
    if let Some(n) = property_coerce_to_number(activation, val)? {
        this.set_visible(activation.context.gc_context, n != 0.0);
    }
    Ok(())
}

fn width<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.width().into())
}

fn set_width<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(val) = property_coerce_to_number(activation, val)? {
        this.set_width(activation.context.gc_context, val);
    }
    Ok(())
}

fn height<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.height().into())
}

fn set_height<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(val) = property_coerce_to_number(activation, val)? {
        this.set_height(activation.context.gc_context, val);
    }
    Ok(())
}

fn rotation<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .rotation(activation.context.gc_context)
        .to_degrees()
        .into())
}

fn set_rotation<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
    degrees: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(mut degrees) = property_coerce_to_number(activation, degrees)? {
        // Normalize into the range of [-180, 180].
        degrees %= 360.0;
        if degrees < -180.0 {
            degrees += 360.0
        } else if degrees > 180.0 {
            degrees -= 360.0
        }
        this.set_rotation(activation.context.gc_context, degrees.to_radians());
    }
    Ok(())
}

fn target<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new(activation.context.gc_context, this.slash_path()).into())
}

fn frames_loaded<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_movie_clip()
        .map(MovieClip::frames_loaded)
        .map(Value::from)
        .unwrap_or(Value::Undefined))
}

fn name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new(activation.context.gc_context, this.name().to_string()).into())
}

fn set_name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let name = val.coerce_to_string(activation)?;
    this.set_name(activation.context.gc_context, &name);
    Ok(())
}

fn drop_target<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "Unimplemented property _droptarget");
    Ok("".into())
}

fn url<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_movie_clip()
        .and_then(|mc| mc.movie())
        .and_then(|mov| mov.url().map(|s| s.to_string()))
        .map(|s| AvmString::new(activation.context.gc_context, s).into())
        .unwrap_or_else(|| "".into()))
}

fn high_quality<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "Unimplemented property _highquality");
    Ok(1.into())
}

fn set_high_quality<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
    _val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    avm_warn!(activation, "Unimplemented property _highquality");
    Ok(())
}

fn focus_rect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "Unimplemented property _focusrect");
    Ok(Value::Null)
}

fn set_focus_rect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
    _val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    avm_warn!(activation, "Unimplemented property _focusrect");
    Ok(())
}

fn sound_buf_time<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "Unimplemented property _soundbuftime");
    Ok(5.into())
}

fn set_sound_buf_time<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
    _val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    avm_warn!(activation, "Unimplemented property _soundbuftime");
    Ok(())
}

fn quality<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "Unimplemented property _quality");
    Ok("HIGH".into())
}

fn set_quality<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
    _val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    avm_warn!(activation, "Unimplemented property _quality");
    Ok(())
}

fn x_mouse<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let local = this.global_to_local(*activation.context.mouse_position);
    Ok(local.0.to_pixels().into())
}

fn y_mouse<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let local = this.global_to_local(*activation.context.mouse_position);
    Ok(local.1.to_pixels().into())
}

fn property_coerce_to_number<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<Option<f64>, Error<'gc>> {
    if value != Value::Undefined && value != Value::Null {
        let n = value.coerce_to_f64(activation)?;
        if n.is_finite() {
            return Ok(Some(n));
        }
    }

    // Invalid value; do not set.
    Ok(None)
}
