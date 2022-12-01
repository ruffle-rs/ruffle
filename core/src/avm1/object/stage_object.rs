//! AVM1 object type to represent objects on the stage.

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property::Attribute;
use crate::avm1::property_map::PropertyMap;
use crate::avm1::{Object, ObjectPtr, ScriptObject, TObject, Value};
use crate::avm_warn;
use crate::context::UpdateContext;
use crate::display_object::{
    DisplayObject, EditText, MovieClip, TDisplayObject, TDisplayObjectContainer,
};
use crate::string::{AvmString, WStr};
use crate::types::Percent;
use gc_arena::{Collect, GcCell, MutationContext};
use std::fmt;

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
        proto: Object<'gc>,
    ) -> Self {
        Self(GcCell::allocate(
            gc_context,
            StageObjectData {
                base: ScriptObject::new(gc_context, Some(proto)),
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
        variable_name: AvmString<'gc>,
    ) {
        self.0
            .write(gc_context)
            .text_field_bindings
            .push(TextFieldBinding {
                text_field,
                variable_name,
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

    fn resolve_path_property(
        self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Value<'gc>> {
        let case_sensitive = activation.is_case_sensitive();
        if name.eq_with_case(b"_root", case_sensitive) {
            return Some(activation.root_object());
        } else if name.eq_with_case(b"_parent", case_sensitive) {
            return Some(
                self.0
                    .read()
                    .display_object
                    .avm1_parent()
                    .map(|dn| dn.object().coerce_to_object(activation))
                    .map(Value::Object)
                    .unwrap_or(Value::Undefined),
            );
        } else if name.eq_with_case(b"_global", case_sensitive) {
            return Some(activation.context.avm1.global_object().into());
        }

        // Resolve level names `_levelN`.
        if let Some(prefix) = name.slice(..6) {
            // `_flash` is a synonym of `_level`, a relic from the earliest Flash versions.
            if prefix.eq_with_case(b"_level", case_sensitive)
                || prefix.eq_with_case(b"_flash", case_sensitive)
            {
                let level_id = Self::parse_level_id(&name[6..]);
                let level = activation
                    .context
                    .stage
                    .child_by_depth(level_id)
                    .map(|o| o.object())
                    .unwrap_or(Value::Undefined);
                return Some(level);
            }
        }

        None
    }

    fn parse_level_id(digits: &WStr) -> i32 {
        // TODO: Use `split_first`?
        let (is_negative, digits) = match digits.get(0) {
            Some(45) => (true, &digits[1..]),
            _ => (false, digits),
        };
        let mut level_id: i32 = 0;
        for digit in digits
            .iter()
            .map_while(|c| char::from_u32(c.into()).and_then(|c| c.to_digit(10)))
        {
            level_id = level_id.wrapping_mul(10);
            level_id = level_id.wrapping_add(digit as i32);
        }
        if is_negative {
            level_id = level_id.wrapping_neg();
        }
        level_id
    }
}

/// A binding from a property of this StageObject to an EditText text field.
#[derive(Collect)]
#[collect(no_drop)]
struct TextFieldBinding<'gc> {
    text_field: EditText<'gc>,
    variable_name: AvmString<'gc>,
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
    fn get_local_stored(
        &self,
        name: impl Into<AvmString<'gc>>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Value<'gc>> {
        let name = name.into();
        let obj = self.0.read();
        let props = activation.context.avm1.display_properties();

        // Property search order for DisplayObjects:
        // 1) Actual properties on the underlying object
        if let Some(value) = obj.base.get_local_stored(name, activation) {
            return Some(value);
        }

        // 2) Path properties such as `_root`, `_parent`, `_levelN` (obeys case sensitivity)
        let magic_property = name.starts_with(b'_');
        if magic_property {
            if let Some(object) = self.resolve_path_property(name, activation) {
                return Some(object);
            }
        }

        // 3) Child display objects with the given instance name
        if let Some(child) = obj
            .display_object
            .as_container()
            .and_then(|o| o.child_by_name(&name, activation.is_case_sensitive()))
        {
            return Some(child.object());
        }

        // 4) Display object properties such as `_x`, `_y` (never case sensitive)
        if magic_property {
            if let Some(property) = props.read().get_by_name(name) {
                return Some(property.get(activation, obj.display_object));
            }
        }

        None
    }

    fn set_local(
        &self,
        name: AvmString<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        let obj = self.0.read();
        let props = activation.context.avm1.display_properties();

        // Check if a text field is bound to this property and update the text if so.
        let case_sensitive = activation.is_case_sensitive();
        for binding in obj.text_field_bindings.iter().filter(|binding| {
            if case_sensitive {
                binding.variable_name == name
            } else {
                binding.variable_name.eq_ignore_case(&name)
            }
        }) {
            binding.text_field.set_html_text(
                &value.coerce_to_string(activation)?,
                &mut activation.context,
            );
        }

        let base = obj.base;
        let display_object = obj.display_object;
        drop(obj);

        if base.has_own_property(activation, name) {
            // 1) Actual properties on the underlying object
            base.set_local(name, value, activation, this)
        } else if let Some(property) = props.read().get_by_name(name) {
            // 2) Display object properties such as _x, _y
            property.set(activation, display_object, value)?;
            Ok(())
        } else {
            // 3) TODO: Prototype
            base.set_local(name, value, activation, this)
        }
    }

    fn call(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Value<'gc>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        self.0.read().base.call(name, activation, this, args)
    }

    fn getter(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Object<'gc>> {
        self.0.read().base.getter(name, activation)
    }

    fn setter(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Object<'gc>> {
        self.0.read().base.setter(name, activation)
    }

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        //TODO: Create a StageObject of some kind
        self.0.read().base.create_bare_object(activation, this)
    }

    fn delete(&self, activation: &mut Activation<'_, 'gc, '_>, name: AvmString<'gc>) -> bool {
        self.0.read().base.delete(activation, name)
    }

    fn proto(&self, activation: &mut Activation<'_, 'gc, '_>) -> Value<'gc> {
        self.0.read().base.proto(activation)
    }

    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: impl Into<AvmString<'gc>>,
        value: Value<'gc>,
        attributes: Attribute,
    ) {
        self.0
            .read()
            .base
            .define_value(gc_context, name, value, attributes)
    }

    fn set_attributes(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: Option<AvmString<'gc>>,
        set_attributes: Attribute,
        clear_attributes: Attribute,
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
        name: AvmString<'gc>,
        get: Object<'gc>,
        set: Option<Object<'gc>>,
        attributes: Attribute,
    ) {
        self.0
            .read()
            .base
            .add_property(gc_context, name, get, set, attributes)
    }

    fn add_property_with_case(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
        get: Object<'gc>,
        set: Option<Object<'gc>>,
        attributes: Attribute,
    ) {
        self.0
            .read()
            .base
            .add_property_with_case(activation, name, get, set, attributes)
    }

    fn call_watcher(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
        value: &mut Value<'gc>,
        this: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        self.0
            .read()
            .base
            .call_watcher(activation, name, value, this)
    }

    fn watch(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
        callback: Object<'gc>,
        user_data: Value<'gc>,
    ) {
        self.0
            .read()
            .base
            .watch(activation, name, callback, user_data);
    }

    fn unwatch(&self, activation: &mut Activation<'_, 'gc, '_>, name: AvmString<'gc>) -> bool {
        self.0.read().base.unwatch(activation, name)
    }

    fn has_property(&self, activation: &mut Activation<'_, 'gc, '_>, name: AvmString<'gc>) -> bool {
        let obj = self.0.read();
        if obj.base.has_property(activation, name) {
            return true;
        }

        let magic_property = name.starts_with(b'_');
        if magic_property
            && activation
                .context
                .avm1
                .display_properties()
                .read()
                .get_by_name(name)
                .is_some()
        {
            return true;
        }

        let case_sensitive = activation.is_case_sensitive();
        if obj
            .display_object
            .as_container()
            .and_then(|o| o.child_by_name(&name, case_sensitive))
            .is_some()
        {
            return true;
        }

        if magic_property && self.resolve_path_property(name, activation).is_some() {
            return true;
        }

        false
    }

    fn has_own_property(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
    ) -> bool {
        // Note that `hasOwnProperty` does NOT return true for child display objects.
        self.0.read().base.has_own_property(activation, name)
    }

    fn has_own_virtual(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
    ) -> bool {
        self.0.read().base.has_own_virtual(activation, name)
    }

    fn is_property_enumerable(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
    ) -> bool {
        self.0.read().base.is_property_enumerable(activation, name)
    }

    fn get_keys(&self, activation: &mut Activation<'_, 'gc, '_>) -> Vec<AvmString<'gc>> {
        // Keys from the underlying object are listed first, followed by
        // child display objects in order from highest depth to lowest depth.
        let obj = self.0.read();
        let mut keys = obj.base.get_keys(activation);

        if let Some(ctr) = obj.display_object.as_container() {
            keys.extend(ctr.iter_render_list().rev().map(|child| child.name()));
        }

        keys
    }

    fn length(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<i32, Error<'gc>> {
        self.0.read().base.length(activation)
    }

    fn set_length(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        length: i32,
    ) -> Result<(), Error<'gc>> {
        self.0.read().base.set_length(activation, length)
    }

    fn has_element(&self, activation: &mut Activation<'_, 'gc, '_>, index: i32) -> bool {
        self.0.read().base.has_element(activation, index)
    }

    fn get_element(&self, activation: &mut Activation<'_, 'gc, '_>, index: i32) -> Value<'gc> {
        self.0.read().base.get_element(activation, index)
    }

    fn set_element(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        index: i32,
        value: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        self.0.read().base.set_element(activation, index, value)
    }

    fn delete_element(&self, activation: &mut Activation<'_, 'gc, '_>, index: i32) -> bool {
        self.0.read().base.delete_element(activation, index)
    }

    fn interfaces(&self) -> Vec<Object<'gc>> {
        self.0.read().base.interfaces()
    }

    fn set_interfaces(&self, gc_context: MutationContext<'gc, '_>, iface_list: Vec<Object<'gc>>) {
        self.0
            .write(gc_context)
            .base
            .set_interfaces(gc_context, iface_list)
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

pub type DisplayGetter<'gc> = fn(&mut Activation<'_, 'gc, '_>, DisplayObject<'gc>) -> Value<'gc>;

pub type DisplaySetter<'gc> =
    fn(&mut Activation<'_, 'gc, '_>, DisplayObject<'gc>, Value<'gc>) -> Result<(), Error<'gc>>;

impl<'gc> DisplayProperty<'gc> {
    pub fn get(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: DisplayObject<'gc>,
    ) -> Value<'gc> {
        (self.get)(activation, this)
    }

    pub fn set(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: DisplayObject<'gc>,
        value: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        if let Some(set) = self.set {
            (set)(activation, this, value)?;
        }
        Ok(())
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
pub struct DisplayPropertyMap<'gc>(PropertyMap<'gc, DisplayProperty<'gc>>);

impl<'gc> DisplayPropertyMap<'gc> {
    /// Creates the display property map.
    pub fn new(gc_context: MutationContext<'gc, '_>) -> GcCell<'gc, DisplayPropertyMap<'gc>> {
        let mut property_map = DisplayPropertyMap(PropertyMap::new());

        // Order is important:
        // should match the SWF specs for GetProperty/SetProperty.
        property_map.add_property("_x".into(), x, Some(set_x));
        property_map.add_property("_y".into(), y, Some(set_y));
        property_map.add_property("_xscale".into(), x_scale, Some(set_x_scale));
        property_map.add_property("_yscale".into(), y_scale, Some(set_y_scale));
        property_map.add_property("_currentframe".into(), current_frame, None);
        property_map.add_property("_totalframes".into(), total_frames, None);
        property_map.add_property("_alpha".into(), alpha, Some(set_alpha));
        property_map.add_property("_visible".into(), visible, Some(set_visible));
        property_map.add_property("_width".into(), width, Some(set_width));
        property_map.add_property("_height".into(), height, Some(set_height));
        property_map.add_property("_rotation".into(), rotation, Some(set_rotation));
        property_map.add_property("_target".into(), target, None);
        property_map.add_property("_framesloaded".into(), frames_loaded, None);
        property_map.add_property("_name".into(), name, Some(set_name));
        property_map.add_property("_droptarget".into(), drop_target, None);
        property_map.add_property("_url".into(), url, None);
        property_map.add_property("_highquality".into(), high_quality, Some(set_high_quality));
        property_map.add_property("_focusrect".into(), focus_rect, Some(set_focus_rect));
        property_map.add_property(
            "_soundbuftime".into(),
            sound_buf_time,
            Some(set_sound_buf_time),
        );
        property_map.add_property("_quality".into(), quality, Some(set_quality));
        property_map.add_property("_xmouse".into(), x_mouse, None);
        property_map.add_property("_ymouse".into(), y_mouse, None);

        GcCell::allocate(gc_context, property_map)
    }

    /// Gets a property slot by name.
    /// Used by `GetMember`, `GetVariable`, `SetMember`, and `SetVariable`.
    pub fn get_by_name(&self, name: AvmString<'gc>) -> Option<&DisplayProperty<'gc>> {
        // Display object properties are case insensitive, regardless of SWF version!?
        // TODO: Another string alloc; optimize this eventually.
        self.0.get(name, false)
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
        name: AvmString<'gc>,
        get: DisplayGetter<'gc>,
        set: Option<DisplaySetter<'gc>>,
    ) {
        let prop = DisplayProperty { get, set };
        self.0.insert(name, prop, false);
    }
}

fn x<'gc>(_activation: &mut Activation<'_, 'gc, '_>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.x().into()
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

fn y<'gc>(_activation: &mut Activation<'_, 'gc, '_>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.y().into()
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

fn x_scale<'gc>(activation: &mut Activation<'_, 'gc, '_>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.scale_x(activation.context.gc_context).percent().into()
}

fn set_x_scale<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(val) = property_coerce_to_number(activation, val)? {
        this.set_scale_x(activation.context.gc_context, Percent::from(val));
    }
    Ok(())
}

fn y_scale<'gc>(activation: &mut Activation<'_, 'gc, '_>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.scale_y(activation.context.gc_context).percent().into()
}

fn set_y_scale<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(val) = property_coerce_to_number(activation, val)? {
        this.set_scale_y(activation.context.gc_context, Percent::from(val));
    }
    Ok(())
}

fn current_frame<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Value<'gc> {
    this.as_movie_clip()
        .map(MovieClip::current_frame)
        .map_or(Value::Undefined, Value::from)
}

fn total_frames<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Value<'gc> {
    this.as_movie_clip()
        .map(MovieClip::total_frames)
        .map_or(Value::Undefined, Value::from)
}

fn alpha<'gc>(_activation: &mut Activation<'_, 'gc, '_>, this: DisplayObject<'gc>) -> Value<'gc> {
    (this.alpha() * 100.0).into()
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

fn visible<'gc>(_activation: &mut Activation<'_, 'gc, '_>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.visible().into()
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

fn width<'gc>(_activation: &mut Activation<'_, 'gc, '_>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.width().into()
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

fn height<'gc>(_activation: &mut Activation<'_, 'gc, '_>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.height().into()
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

fn rotation<'gc>(activation: &mut Activation<'_, 'gc, '_>, this: DisplayObject<'gc>) -> Value<'gc> {
    let degrees: f64 = this.rotation(activation.context.gc_context).into();
    degrees.into()
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
        this.set_rotation(activation.context.gc_context, degrees.into());
    }
    Ok(())
}

fn target<'gc>(activation: &mut Activation<'_, 'gc, '_>, this: DisplayObject<'gc>) -> Value<'gc> {
    AvmString::new(activation.context.gc_context, this.slash_path()).into()
}

fn frames_loaded<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Value<'gc> {
    this.as_movie_clip()
        .map(MovieClip::frames_loaded)
        .map_or(Value::Undefined, Value::from)
}

fn name<'gc>(_activation: &mut Activation<'_, 'gc, '_>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.name().into()
}

fn set_name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let name = val.coerce_to_string(activation)?;
    this.set_name(activation.context.gc_context, name);
    Ok(())
}

fn drop_target<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: DisplayObject<'gc>,
) -> Value<'gc> {
    this.as_movie_clip()
        .and_then(|mc| mc.drop_target())
        .map_or_else(
            || {
                if activation.swf_version() < 6 {
                    Value::Undefined
                } else {
                    "".into()
                }
            },
            |drop_target| {
                AvmString::new(activation.context.gc_context, drop_target.slash_path()).into()
            },
        )
}

fn url<'gc>(activation: &mut Activation<'_, 'gc, '_>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.as_movie_clip()
        .and_then(|mc| mc.movie())
        .and_then(|mov| mov.url().map(|url| url.to_string()))
        .map_or_else(
            || "".into(),
            |s| AvmString::new_utf8(activation.context.gc_context, s).into(),
        )
}

fn high_quality<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
) -> Value<'gc> {
    use crate::display_object::StageQuality;
    let quality = match activation.context.stage.quality() {
        StageQuality::Best => 2,
        StageQuality::High => 1,
        _ => 0,
    };
    quality.into()
}

fn set_high_quality<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    use crate::display_object::StageQuality;
    let val = val.coerce_to_f64(activation)?;
    if !val.is_nan() {
        // 0 -> Low, 1 -> High, 2 -> Best, but with some odd rules for non-integers.
        let quality = if val > 1.5 {
            StageQuality::Best
        } else if val == 0.0 {
            StageQuality::Low
        } else {
            StageQuality::High
        };
        activation
            .context
            .stage
            .set_quality(activation.context.gc_context, quality);
    }
    Ok(())
}

fn focus_rect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
) -> Value<'gc> {
    avm_warn!(activation, "Unimplemented property _focusrect");
    Value::Null
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
) -> Value<'gc> {
    activation.context.audio_manager.stream_buffer_time().into()
}

fn set_sound_buf_time<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    avm_warn!(activation, "_soundbuftime is currently ignored by Ruffle");
    if let Some(val) = property_coerce_to_i32(activation, val)? {
        activation.context.audio_manager.set_stream_buffer_time(val);
    }
    Ok(())
}

fn quality<'gc>(activation: &mut Activation<'_, 'gc, '_>, _this: DisplayObject<'gc>) -> Value<'gc> {
    let quality = activation.context.stage.quality().into_avm_str();
    quality.into()
}

fn set_quality<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Ok(quality) = val.coerce_to_string(activation)?.parse() {
        activation
            .context
            .stage
            .set_quality(activation.context.gc_context, quality);
    }
    Ok(())
}

fn x_mouse<'gc>(activation: &mut Activation<'_, 'gc, '_>, this: DisplayObject<'gc>) -> Value<'gc> {
    let (local_x, _) = this.global_to_local(*activation.context.mouse_position);
    local_x.to_pixels().into()
}

fn y_mouse<'gc>(activation: &mut Activation<'_, 'gc, '_>, this: DisplayObject<'gc>) -> Value<'gc> {
    let (_, local_y) = this.global_to_local(*activation.context.mouse_position);
    local_y.to_pixels().into()
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

/// Coerces `value` to `i32` for use by a stage object property.
///
/// Values out of range of `i32` will be clamped to `i32::MIN`. Returns `None` if the value is
/// invalid (NaN, null, or undefined).
fn property_coerce_to_i32<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<Option<i32>, Error<'gc>> {
    let n = value.coerce_to_f64(activation)?;
    let ret = if n.is_nan() {
        // NaN/undefined/null are invalid values; do not set.
        None
    } else if n >= i32::MIN as f64 && n <= i32::MAX as f64 {
        Some(n as i32)
    } else {
        // Out of range of i32; snaps to `i32::MIN`.
        Some(i32::MIN)
    };

    Ok(ret)
}
