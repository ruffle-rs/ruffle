//! AVM1 object type to represent objects on the stage.

use crate::avm1::activation::Activation;
use crate::avm1::clamp::Clamp;
use crate::avm1::error::Error;
use crate::avm1::property_map::PropertyMap;
use crate::avm1::{Object, ObjectPtr, ScriptObject, TObject, Value};
use crate::avm_warn;
use crate::context::UpdateContext;
use crate::display_object::{
    DisplayObject, EditText, MovieClip, TDisplayObject, TDisplayObjectContainer, TInteractiveObject,
};
use crate::string::{AvmString, WStr};
use crate::types::Percent;
use gc_arena::{Collect, GcCell, GcWeakCell, Mutation};
use std::fmt;
use swf::Twips;

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
    pub display_object: DisplayObject<'gc>,

    text_field_bindings: Vec<TextFieldBinding<'gc>>,
}

impl<'gc> StageObject<'gc> {
    /// Create a weak reference to the underlying data of this `StageObject`
    pub fn as_weak(&self) -> GcWeakCell<'gc, StageObjectData<'gc>> {
        GcCell::downgrade(self.0)
    }

    /// Create a stage object for a given display node.
    pub fn for_display_object(
        gc_context: &Mutation<'gc>,
        display_object: DisplayObject<'gc>,
        proto: Object<'gc>,
    ) -> Self {
        Self(GcCell::new(
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
        gc_context: &Mutation<'gc>,
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
    pub fn clear_text_field_binding(self, gc_context: &Mutation<'gc>, text_field: EditText<'gc>) {
        self.0
            .write(gc_context)
            .text_field_bindings
            .retain(|binding| !DisplayObject::ptr_eq(text_field.into(), binding.text_field.into()));
    }

    /// Clears all text field bindings from this stage object, and places the textfields on the unbound list.
    /// This is called when the object is removed from the stage.
    pub fn unregister_text_field_bindings(self, context: &mut UpdateContext<'gc>) {
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
        activation: &mut Activation<'_, 'gc>,
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
                    .get_level(level_id)
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("StageObject")
            .field("ptr", &self.0.as_ptr())
            .field("display_object", &this.display_object)
            .finish()
    }
}

impl<'gc> TObject<'gc> for StageObject<'gc> {
    fn raw_script_object(&self) -> ScriptObject<'gc> {
        self.0.read().base
    }

    fn get_local_stored(
        &self,
        name: impl Into<AvmString<'gc>>,
        activation: &mut Activation<'_, 'gc>,
        is_slash_path: bool,
    ) -> Option<Value<'gc>> {
        let name = name.into();
        let obj = self.0.read();

        // Property search order for DisplayObjects:
        // 1) Actual properties on the underlying object
        if let Some(value) = obj.base.get_local_stored(name, activation, is_slash_path) {
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
            return if is_slash_path {
                Some(child.object())
            // If an object doesn't have an object representation, e.g. Graphic, then trying to access it
            // Returns the parent instead
            } else if let crate::display_object::DisplayObject::Graphic(_) = child {
                child.parent().map(|p| p.object())
            } else {
                Some(child.object())
            };
        }

        // 4) Display object properties such as `_x`, `_y` (never case sensitive)
        if magic_property {
            if let Some(property) = activation
                .context
                .avm1
                .display_properties()
                .get_by_name(name)
                .copied()
            {
                return Some(property.get(activation, obj.display_object));
            }
        }

        None
    }

    fn set_local(
        &self,
        name: AvmString<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
        this: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        let obj = self.0.read();

        // Check if a text field is bound to this property and update the text if so.
        let case_sensitive = activation.is_case_sensitive();
        for binding in obj.text_field_bindings.iter().filter(|binding| {
            if case_sensitive {
                binding.variable_name == name
            } else {
                binding.variable_name.eq_ignore_case(&name)
            }
        }) {
            binding
                .text_field
                .set_html_text(&value.coerce_to_string(activation)?, activation.context);
        }

        let base = obj.base;
        let display_object = obj.display_object;
        drop(obj);

        if base.has_own_property(activation, name) {
            // 1) Actual properties on the underlying object
            base.set_local(name, value, activation, this)
        } else if let Some(property) = activation
            .context
            .avm1
            .display_properties()
            .get_by_name(name)
            .copied()
        {
            // 2) Display object properties such as _x, _y
            property.set(activation, display_object, value)
        } else {
            // 3) TODO: Prototype
            base.set_local(name, value, activation, this)
        }
    }

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        //TODO: Create a StageObject of some kind
        self.0.read().base.create_bare_object(activation, this)
    }

    // Note that `hasOwnProperty` does NOT return true for child display objects.
    fn has_property(&self, activation: &mut Activation<'_, 'gc>, name: AvmString<'gc>) -> bool {
        let obj = self.0.read();

        if !obj.display_object.avm1_removed() && obj.base.has_property(activation, name) {
            return true;
        }

        let magic_property = name.starts_with(b'_');
        if magic_property
            && activation
                .context
                .avm1
                .display_properties()
                .get_by_name(name)
                .is_some()
        {
            return true;
        }

        let case_sensitive = activation.is_case_sensitive();

        if !obj.display_object.avm1_removed()
            && obj
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

    fn get_keys(
        &self,
        activation: &mut Activation<'_, 'gc>,
        include_hidden: bool,
    ) -> Vec<AvmString<'gc>> {
        // Keys from the underlying object are listed first, followed by
        // child display objects in order from highest depth to lowest depth.
        let obj = self.0.read();
        let mut keys = obj.base.get_keys(activation, include_hidden);

        if let Some(ctr) = obj.display_object.as_container() {
            // Button/MovieClip children are included in key list.
            for child in ctr.iter_render_list().rev() {
                if child.as_interactive().is_some() {
                    keys.push(child.name());
                }
            }
        }

        keys
    }

    /// Get the underlying stage object, if it exists.
    fn as_stage_object(&self) -> Option<StageObject<'gc>> {
        Some(*self)
    }

    fn as_display_object(&self) -> Option<DisplayObject<'gc>> {
        Some(self.0.read().display_object)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.read().base.as_ptr()
    }
}

/// Properties shared by display objects in AVM1, such as _x and _y.
/// These are only accessible for movie clips, buttons, and text fields (any others?)
/// These exist outside the global or prototype machinery. Instead, they are
/// "special" properties stored in a separate map that display objects look at in addition
/// to normal property lookup.
/// The map of property names to display object getts/setters.
#[derive(Copy, Clone, Collect)]
#[collect(require_static)]
pub struct DisplayProperty {
    get: DisplayGetter,
    set: Option<DisplaySetter>,
}

pub type DisplayGetter = for<'gc> fn(&mut Activation<'_, 'gc>, DisplayObject<'gc>) -> Value<'gc>;
pub type DisplaySetter =
    for<'gc> fn(&mut Activation<'_, 'gc>, DisplayObject<'gc>, Value<'gc>) -> Result<(), Error<'gc>>;

impl<'gc> DisplayProperty {
    pub fn get(
        &self,
        activation: &mut Activation<'_, 'gc>,
        this: DisplayObject<'gc>,
    ) -> Value<'gc> {
        (self.get)(activation, this)
    }

    pub fn set(
        &self,
        activation: &mut Activation<'_, 'gc>,
        this: DisplayObject<'gc>,
        value: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        if let Some(set) = self.set {
            (set)(activation, this, value)?;
        }
        Ok(())
    }

    pub fn is_read_only(&self) -> bool {
        self.set.is_none()
    }
}

/// The map from key/index to function pointers for special display object properties.
#[derive(Collect)]
#[collect(no_drop)]
pub struct DisplayPropertyMap<'gc>(PropertyMap<'gc, DisplayProperty>);

impl<'gc> DisplayPropertyMap<'gc> {
    /// Creates the display property map.
    pub fn new() -> Self {
        let mut property_map = Self(PropertyMap::new());

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

        property_map
    }

    /// Gets a property slot by name.
    /// Used by `GetMember`, `GetVariable`, `SetMember`, and `SetVariable`.
    pub fn get_by_name(&self, name: AvmString<'gc>) -> Option<&DisplayProperty> {
        // Display object properties are case insensitive, regardless of SWF version!?
        // TODO: Another string alloc; optimize this eventually.
        self.0.get(name, false)
    }

    /// Gets a property slot by SWF4 index.
    /// The order is defined by the SWF specs.
    /// Used by `GetProperty`/`SetProperty`.
    /// SWF19 pp. 85-86
    pub fn get_by_index(&self, index: usize) -> Option<&DisplayProperty> {
        self.0.get_index(index)
    }

    fn add_property(
        &mut self,
        name: AvmString<'gc>,
        get: DisplayGetter,
        set: Option<DisplaySetter>,
    ) {
        let prop = DisplayProperty { get, set };
        self.0.insert(name, prop, false);
    }
}

impl Default for DisplayPropertyMap<'_> {
    fn default() -> Self {
        Self::new()
    }
}

fn x<'gc>(_activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.x().to_pixels().into()
}

fn set_x<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(x) = property_coerce_to_number(activation, val)? {
        this.set_x(activation.context.gc_context, Twips::from_pixels(x));
    }
    Ok(())
}

fn y<'gc>(_activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.y().to_pixels().into()
}

fn set_y<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(y) = property_coerce_to_number(activation, val)? {
        this.set_y(activation.context.gc_context, Twips::from_pixels(y));
    }
    Ok(())
}

fn x_scale<'gc>(activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.scale_x(activation.context.gc_context).percent().into()
}

fn set_x_scale<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(val) = property_coerce_to_number(activation, val)? {
        this.set_scale_x(activation.context.gc_context, Percent::from(val));
    }
    Ok(())
}

fn y_scale<'gc>(activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.scale_y(activation.context.gc_context).percent().into()
}

fn set_y_scale<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(val) = property_coerce_to_number(activation, val)? {
        this.set_scale_y(activation.context.gc_context, Percent::from(val));
    }
    Ok(())
}

fn current_frame<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
) -> Value<'gc> {
    this.as_movie_clip()
        .map(MovieClip::current_frame)
        .map_or(Value::Undefined, Value::from)
}

fn total_frames<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
) -> Value<'gc> {
    this.as_movie_clip()
        .map(MovieClip::total_frames)
        .map_or(Value::Undefined, Value::from)
}

fn alpha<'gc>(_activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    (this.alpha() * 100.0).into()
}

fn set_alpha<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(val) = property_coerce_to_number(activation, val)? {
        this.set_alpha(activation.context.gc_context, val / 100.0);
    }
    Ok(())
}

fn visible<'gc>(_activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.visible().into()
}

fn set_visible<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    // Because this property dates to the era of Flash 4, this is actually coerced to an integer.
    // `_visible = "false";` coerces to NaN and has no effect.
    if let Some(n) = property_coerce_to_number(activation, val)? {
        this.set_visible(activation.context, n != 0.0);
    }
    Ok(())
}

fn width<'gc>(_activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.width().into()
}

fn set_width<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(val) = property_coerce_to_number(activation, val)? {
        this.set_width(activation.context, val);
    }
    Ok(())
}

fn height<'gc>(_activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.height().into()
}

fn set_height<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(val) = property_coerce_to_number(activation, val)? {
        this.set_height(activation.context, val);
    }
    Ok(())
}

fn rotation<'gc>(activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    let degrees: f64 = this.rotation(activation.context.gc_context).into();
    degrees.into()
}

fn set_rotation<'gc>(
    activation: &mut Activation<'_, 'gc>,
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

fn target<'gc>(activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    AvmString::new(activation.context.gc_context, this.slash_path()).into()
}

fn frames_loaded<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
) -> Value<'gc> {
    this.as_movie_clip()
        .map(MovieClip::frames_loaded_for_avm)
        .map_or(Value::Undefined, Value::from)
}

fn name<'gc>(_activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.name().into()
}

fn set_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let name = val.coerce_to_string(activation)?;
    this.set_name(activation.context.gc_context, name);
    Ok(())
}

fn drop_target<'gc>(activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    match this.as_movie_clip().and_then(|mc| mc.drop_target()) {
        Some(target) => AvmString::new(activation.gc(), target.slash_path()).into(),
        None if activation.swf_version() < 6 => Value::Undefined,
        None => activation.strings().empty().into(),
    }
}

fn url<'gc>(activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    match this.as_movie_clip() {
        Some(mc) => AvmString::new_utf8(activation.gc(), mc.movie().url()).into(),
        None => activation.strings().empty().into(),
    }
}

fn high_quality<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: DisplayObject<'gc>,
) -> Value<'gc> {
    use ruffle_render::quality::StageQuality;
    let quality = match activation.context.stage.quality() {
        StageQuality::Best => 2,
        StageQuality::High => 1,
        _ => 0,
    };
    quality.into()
}

fn set_high_quality<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    use ruffle_render::quality::StageQuality;
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
            .set_quality(activation.context, quality);
    }
    Ok(())
}

fn refers_to_stage_focus_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
) -> bool {
    activation.swf_version() <= 5 || this.parent().is_some_and(|p| p.as_stage().is_some())
}

fn focus_rect<'gc>(activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    if refers_to_stage_focus_rect(activation, this) {
        let val = activation.context.stage.stage_focus_rect();
        if activation.swf_version() <= 5 {
            Value::Number(if val { 1.0 } else { 0.0 })
        } else {
            Value::Bool(val)
        }
    } else if let Some(obj) = this.as_interactive() {
        match obj.focus_rect() {
            Some(val) => Value::Bool(val),
            None => Value::Null,
        }
    } else {
        Value::Undefined
    }
}

fn set_focus_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if refers_to_stage_focus_rect(activation, this) {
        let val = match val {
            Value::Undefined | Value::Null => {
                // undefined & null are ignored
                return Ok(());
            }
            Value::Object(_) => false,
            _ => val.coerce_to_f64(activation)? != 0.0,
        };
        activation
            .context
            .stage
            .set_stage_focus_rect(activation.context.gc(), val);
    } else if let Some(obj) = this.as_interactive() {
        let val = match val {
            Value::Undefined | Value::Null => None,
            _ => Some(val.as_bool(activation.swf_version())),
        };
        obj.set_focus_rect(activation.context.gc(), val);
    }
    Ok(())
}

fn sound_buf_time<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: DisplayObject<'gc>,
) -> Value<'gc> {
    activation.context.audio_manager.stream_buffer_time().into()
}

fn set_sound_buf_time<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    avm_warn!(activation, "_soundbuftime is currently ignored by Ruffle");
    let val = val.coerce_to_f64(activation)?;
    // NaN/undefined/null are invalid values; do not set.
    if !val.is_nan() {
        activation
            .context
            .audio_manager
            .set_stream_buffer_time(val.clamp_to_i32());
    }
    Ok(())
}

fn quality<'gc>(activation: &mut Activation<'_, 'gc>, _this: DisplayObject<'gc>) -> Value<'gc> {
    let quality = activation.context.stage.quality().into_avm_str();
    quality.into()
}

fn set_quality<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Ok(quality) = val.coerce_to_string(activation)?.parse() {
        activation
            .context
            .stage
            .set_quality(activation.context, quality);
    }
    Ok(())
}

fn x_mouse<'gc>(activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    let local = this.local_mouse_position(activation.context);
    local.x.to_pixels().into()
}

fn y_mouse<'gc>(activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    let local = this.local_mouse_position(activation.context);
    local.y.to_pixels().into()
}

fn property_coerce_to_number<'gc>(
    activation: &mut Activation<'_, 'gc>,
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

/// Coerces a value according to the property index.
/// Used by `SetProperty`.
pub fn action_property_coerce<'gc>(
    activation: &mut Activation<'_, 'gc>,
    index: usize,
    value: Value<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(match index {
        // Coerce to a number. This affects the following properties (including some which have no setter):
        // _x, _y, _xscale, _yscale, _currentframe, _totalframes, _alpha, _visible, _width, _height, _rotation, _framesloaded.
        0..=10 | 12 => {
            if let Some(value_to_number) = property_coerce_to_number(activation, value)? {
                value_to_number.into()
            } else {
                value
            }
        }
        // Coerce to a f64. This affects the following properties (including some which have no setter):
        // _highquality, _soundbuftime, _xmouse, _ymouse.
        16 | 18 | 20..=21 => value.coerce_to_f64(activation)?.into(),
        // Coerce to a string. This affects the following properties:
        // _name, _quality.
        13 | 19 => value.coerce_to_string(activation)?.into(),
        // No coercion. This affects the following properties:
        // _target, _droptarget, _url, _focusrect.
        _ => value,
    })
}
