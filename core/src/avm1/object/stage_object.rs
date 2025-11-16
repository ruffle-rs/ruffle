//! DisplayObject-specific AVM1 operations.

use crate::avm1::activation::Activation;
use crate::avm1::clamp::Clamp;
use crate::avm1::error::Error;
use crate::avm1::property_map::PropertyMap;
use crate::avm1::Value;
use crate::avm_warn;
use crate::display_object::{
    DisplayObject, MovieClip, TDisplayObject, TDisplayObjectContainer, TInteractiveObject,
};
use crate::string::{AvmString, StringContext, WStr};
use crate::types::Percent;
use gc_arena::Collect;
use ruffle_macros::istr;
use smallvec::SmallVec;
use swf::Twips;

pub fn get_property<'gc>(
    dobj: DisplayObject<'gc>,
    name: AvmString<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Option<Value<'gc>> {
    // Property search order for DisplayObjects:

    // 1) Path properties such as `_root`, `_parent`, `_levelN` (obeys case sensitivity)
    let magic_property = name.starts_with(b'_');
    if magic_property {
        if let Some(object) = resolve_path_property(dobj, name, activation) {
            return Some(object);
        }
    }

    // 2) Child display objects with the given instance name
    if let Some(child) = dobj
        .as_container()
        .and_then(|o| o.child_by_name(&name, activation.is_case_sensitive()))
    {
        let value = child
            .object1()
            // If an object doesn't have an object representation, e.g. Graphic,
            // then trying to access it returns the parent instead
            .or_else(|| child.parent().and_then(|p| p.object1()))
            .map_or(Value::Undefined, Value::from);
        return Some(value);
    }

    // 3) Display object properties such as `_x`, `_y` (never case sensitive)
    if magic_property {
        if let Some(property) = activation
            .context
            .avm1
            .display_properties()
            .get_by_name(name)
        {
            return Some(property.get(activation, dobj));
        }
    }

    None
}

/// Notify any bound text fields of a property change.
///
/// This should be called every time a property is set on a AVM1 object.
pub fn notify_property_change<'gc>(
    dobj: DisplayObject<'gc>,
    property_name: AvmString<'gc>,
    value: Value<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<(), Error<'gc>> {
    // Temporary vec to avoid double-borrow errors.
    let mut text_fields = SmallVec::<[_; 1]>::new();

    // Check if a text field is bound to this property and update the text if so.
    let case_sensitive = activation.is_case_sensitive();
    if let Some(bindings) = dobj.avm1_text_field_bindings() {
        for binding in bindings.iter().filter(|binding| {
            if case_sensitive {
                binding.variable_name == property_name
            } else {
                binding.variable_name.eq_ignore_case(&property_name)
            }
        }) {
            text_fields.push(binding.text_field);
        }
    }

    for tf in text_fields {
        tf.set_html_text(&value.coerce_to_string(activation)?, activation.context);
    }

    Ok(())
}

pub fn has_display_object_property<'gc>(
    dobj: DisplayObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
    name: AvmString<'gc>,
) -> bool {
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

    if dobj
        .as_container()
        .and_then(|o| o.child_by_name(&name, case_sensitive))
        .is_some()
    {
        return true;
    }

    if magic_property && resolve_path_property(dobj, name, activation).is_some() {
        return true;
    }

    false
}

pub fn enumerate_keys<'gc>(dobj: DisplayObject<'gc>, keys: &mut Vec<AvmString<'gc>>) {
    // Keys from the underlying object are listed first, followed by
    // child display objects in order from highest depth to lowest depth.
    if let Some(ctr) = dobj.as_container() {
        // Button/MovieClip children are included in key list.
        for child in ctr.iter_render_list().rev() {
            // All named DOs are included in the list, even if they're not
            // accessible by AVM1 code (e.g. `MorphShape`)
            if let Some(name) = child.name() {
                keys.push(name);
            }
        }
    }
}

fn resolve_path_property<'gc>(
    dobj: DisplayObject<'gc>,
    name: AvmString<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Option<Value<'gc>> {
    let case_sensitive = activation.is_case_sensitive();
    if name.eq_with_case(b"_root", case_sensitive) {
        return Some(dobj.avm1_root().object1_or_undef());
    } else if name.eq_with_case(b"_parent", case_sensitive) {
        return Some(
            dobj.avm1_parent()
                .map(|dn| dn.object1_or_bare(activation.gc()))
                .map(Value::Object)
                .unwrap_or(Value::Undefined),
        );
    } else if activation.swf_version() > 5 && name.eq_with_case(b"_global", case_sensitive) {
        // _global is available only in SWF6+
        return Some(activation.global_object().into());
    }

    // Resolve level names `_levelN`.
    if let Some(prefix) = name.slice(..6) {
        // `_flash` is a synonym of `_level`, a relic from the earliest Flash versions.
        if prefix.eq_with_case(b"_level", case_sensitive)
            || prefix.eq_with_case(b"_flash", case_sensitive)
        {
            let level_id = parse_level_id(&name[6..]);
            let level = activation
                .get_level(level_id)
                .map(|o| o.object1_or_undef())
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

/// Properties shared by display objects in AVM1, such as _x and _y.
/// These are only accessible for movie clips, buttons, text fields, and videos
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
    pub fn new(context: &mut StringContext<'gc>) -> Self {
        let mut property_map = Self(PropertyMap::new());

        // Don't use `istr!` here, this is only done once during AVM1 initialization.
        for &(name, getter, setter) in Self::PROPERTIES {
            let name = context.intern_static(WStr::from_units(name));
            property_map.add_property(name.into(), getter, setter);
        }

        property_map
    }

    // Order is important:
    // should match the SWF specs for GetProperty/SetProperty.
    const PROPERTIES: &'static [(&'static [u8], DisplayGetter, Option<DisplaySetter>)] = &[
        (b"_x", x, Some(set_x)),
        (b"_y", y, Some(set_y)),
        (b"_xscale", x_scale, Some(set_x_scale)),
        (b"_yscale", y_scale, Some(set_y_scale)),
        (b"_currentframe", current_frame, None),
        (b"_totalframes", total_frames, None),
        (b"_alpha", alpha, Some(set_alpha)),
        (b"_visible", visible, Some(set_visible)),
        (b"_width", width, Some(set_width)),
        (b"_height", height, Some(set_height)),
        (b"_rotation", rotation, Some(set_rotation)),
        (b"_target", target, None),
        (b"_framesloaded", frames_loaded, None),
        (b"_name", name, Some(set_name)),
        (b"_droptarget", drop_target, None),
        (b"_url", url, None),
        (b"_highquality", high_quality, Some(set_high_quality)),
        (b"_focusrect", focus_rect, Some(set_focus_rect)),
        (b"_soundbuftime", sound_buf_time, Some(set_sound_buf_time)),
        (b"_quality", quality, Some(set_quality)),
        (b"_xmouse", x_mouse, None),
        (b"_ymouse", y_mouse, None),
    ];

    /// Gets a property slot by name.
    /// Used by `GetMember`, `GetVariable`, `SetMember`, and `SetVariable`.
    pub fn get_by_name(&self, name: AvmString<'gc>) -> Option<DisplayProperty> {
        // Display object properties are case insensitive, regardless of SWF version!?
        self.0.get(name, false).copied()
    }

    /// Gets a property slot by SWF4 index.
    /// The order is defined by the SWF specs.
    /// Used by `GetProperty`/`SetProperty`.
    /// SWF19 pp. 85-86
    pub fn get_by_index(&self, index: usize) -> Option<DisplayProperty> {
        self.0.get_index(index).copied()
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

fn x<'gc>(_activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.x().to_pixels().into()
}

fn set_x<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(x) = property_coerce_to_number(activation, val)? {
        this.set_x(Twips::from_pixels(x));
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
        this.set_y(Twips::from_pixels(y));
    }
    Ok(())
}

fn x_scale<'gc>(_activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.scale_x().percent().into()
}

fn set_x_scale<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(val) = property_coerce_to_number(activation, val)? {
        this.set_scale_x(Percent::from(val));
    }
    Ok(())
}

fn y_scale<'gc>(_activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.scale_y().percent().into()
}

fn set_y_scale<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(val) = property_coerce_to_number(activation, val)? {
        this.set_scale_y(Percent::from(val));
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
        .map(MovieClip::header_frames)
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
        this.set_alpha(val / 100.0);
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

fn rotation<'gc>(_activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    let degrees: f64 = this.rotation().into();
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
        this.set_rotation(degrees.into());
    }
    Ok(())
}

fn target<'gc>(activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    AvmString::new(activation.gc(), this.slash_path()).into()
}

fn frames_loaded<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
) -> Value<'gc> {
    if let Some(mc) = this.as_movie_clip() {
        return mc.frames_loaded().min(mc.header_frames() as i32).into();
    }
    Value::Undefined
}

fn name<'gc>(activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    this.name().unwrap_or_else(|| istr!("")).into()
}

fn set_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: DisplayObject<'gc>,
    val: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let name = val.coerce_to_string(activation)?;
    this.set_name(activation.gc(), name);
    Ok(())
}

fn drop_target<'gc>(activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    match this.as_movie_clip().and_then(|mc| mc.drop_target()) {
        Some(target) => AvmString::new(activation.gc(), target.slash_path()).into(),
        None if activation.swf_version() < 6 => Value::Undefined,
        None => istr!("").into(),
    }
}

fn url<'gc>(activation: &mut Activation<'_, 'gc>, this: DisplayObject<'gc>) -> Value<'gc> {
    match this.as_movie_clip() {
        Some(mc) => AvmString::new_utf8(activation.gc(), mc.movie().url()).into(),
        None => istr!("").into(),
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
        activation.context.stage.set_stage_focus_rect(val);
    } else if let Some(obj) = this.as_interactive() {
        let val = match val {
            Value::Undefined | Value::Null => None,
            _ => Some(val.as_bool(activation.swf_version())),
        };
        obj.set_focus_rect(val);
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
    AvmString::new_utf8(activation.gc(), quality).into()
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
