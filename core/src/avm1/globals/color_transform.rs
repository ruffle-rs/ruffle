//! flash.geom.ColorTransform object

use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, Error, Object, ScriptObject, TObject, Value};
use crate::string::{AvmString, StringContext};
use gc_arena::{Collect, GcCell};
use swf::{ColorTransform, Fixed8};

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct ColorTransformObject {
    red_multiplier: f64,
    green_multiplier: f64,
    blue_multiplier: f64,
    alpha_multiplier: f64,
    red_offset: f64,
    green_offset: f64,
    blue_offset: f64,
    alpha_offset: f64,
}

impl<'gc> ColorTransformObject {
    pub const IDENTITY: Self = Self {
        red_multiplier: 1.0,
        green_multiplier: 1.0,
        blue_multiplier: 1.0,
        alpha_multiplier: 1.0,
        red_offset: 0.0,
        green_offset: 0.0,
        blue_offset: 0.0,
        alpha_offset: 0.0,
    };

    pub fn construct(
        activation: &mut Activation<'_, 'gc>,
        color_transform: &ColorTransform,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let args = [
            color_transform.r_multiply.to_f64().into(),
            color_transform.g_multiply.to_f64().into(),
            color_transform.b_multiply.to_f64().into(),
            color_transform.a_multiply.to_f64().into(),
            color_transform.r_add.into(),
            color_transform.g_add.into(),
            color_transform.b_add.into(),
            color_transform.a_add.into(),
        ];
        let constructor = activation
            .context
            .avm1
            .prototypes()
            .color_transform_constructor;
        constructor.construct(activation, &args)
    }

    pub fn cast(value: Value<'gc>) -> Option<GcCell<'gc, Self>> {
        if let Value::Object(object) = value {
            if let NativeObject::ColorTransform(color_transform) = object.native() {
                return Some(color_transform);
            }
        }
        None
    }
}

impl From<ColorTransformObject> for ColorTransform {
    fn from(object: ColorTransformObject) -> Self {
        Self {
            r_multiply: Fixed8::from_f64(object.red_multiplier),
            g_multiply: Fixed8::from_f64(object.green_multiplier),
            b_multiply: Fixed8::from_f64(object.blue_multiplier),
            a_multiply: Fixed8::from_f64(object.alpha_multiplier),
            r_add: object.red_offset as i16,
            g_add: object.green_offset as i16,
            b_add: object.blue_offset as i16,
            a_add: object.alpha_offset as i16,
        }
    }
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "alphaMultiplier" => property(get_alpha_multiplier, set_alpha_multiplier);
    "redMultiplier" => property(get_red_multiplier, set_red_multiplier);
    "greenMultiplier" => property(get_green_multiplier, set_green_multiplier);
    "blueMultiplier" => property(get_blue_multiplier, set_blue_multiplier);
    "alphaOffset" => property(get_alpha_offset, set_alpha_offset);
    "redOffset" => property(get_red_offset, set_red_offset);
    "greenOffset" => property(get_green_offset, set_green_offset);
    "blueOffset" => property(get_blue_offset, set_blue_offset);
    "rgb" => property(get_rgb, set_rgb);
    "concat" => method(concat);
    "toString" => method(to_string);
};

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let color_transform = match args {
        _ if args.len() >= 8 => {
            let mut values = [0.0; 8];
            for (arg, value) in args.iter().zip(&mut values) {
                *value = arg.coerce_to_f64(activation)?;
            }
            let [red_multiplier, green_multiplier, blue_multiplier, alpha_multiplier, red_offset, green_offset, blue_offset, alpha_offset] =
                values;
            ColorTransformObject {
                red_multiplier,
                green_multiplier,
                blue_multiplier,
                alpha_multiplier,
                red_offset,
                green_offset,
                blue_offset,
                alpha_offset,
            }
        }
        [color_transform] => {
            if let Some(color_transform) = ColorTransformObject::cast(*color_transform) {
                color_transform.read().clone()
            } else {
                ColorTransformObject::IDENTITY
            }
        }
        _ => ColorTransformObject::IDENTITY,
    };
    this.set_native(
        activation.context.gc_context,
        NativeObject::ColorTransform(GcCell::new(activation.context.gc_context, color_transform)),
    );
    Ok(this.into())
}

fn get_rgb<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(color_transform) = ColorTransformObject::cast(this.into()) {
        let color_transform = color_transform.read();
        let rgb = ((color_transform.red_offset as i32) << 16)
            | ((color_transform.green_offset as i32) << 8)
            | (color_transform.blue_offset as i32);
        Ok(rgb.into())
    } else {
        Ok(Value::Undefined)
    }
}

fn set_rgb<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(color_transform) = ColorTransformObject::cast(this.into()) {
        if let [rgb, ..] = args {
            let rgb = rgb.coerce_to_u32(activation)?;
            let [b, g, r, _] = rgb.to_le_bytes();
            let mut color_transform = color_transform.write(activation.context.gc_context);
            color_transform.red_multiplier = 0.0;
            color_transform.green_multiplier = 0.0;
            color_transform.blue_multiplier = 0.0;
            color_transform.red_offset = r.into();
            color_transform.green_offset = g.into();
            color_transform.blue_offset = b.into();
        }
    }

    Ok(Value::Undefined)
}

macro_rules! color_transform_value_accessor {
    ($([$field: ident, $getter: ident, $setter: ident],)*) => {
        $(
            fn $getter<'gc>(
                _activation: &mut Activation<'_, 'gc>,
                this: Object<'gc>,
                _args: &[Value<'gc>],
            ) -> Result<Value<'gc>, Error<'gc>> {
                if let Some(color_transform) = ColorTransformObject::cast(this.into()) {
                    Ok(color_transform.read().$field.into())
                } else {
                    Ok(Value::Undefined)
                }
            }

            fn $setter<'gc>(
                activation: &mut Activation<'_, 'gc>,
                this: Object<'gc>,
                args: &[Value<'gc>],
            ) -> Result<Value<'gc>, Error<'gc>> {
                if let Some(color_transform) = ColorTransformObject::cast(this.into()) {
                    if let [value, ..] = args {
                        let value = value.coerce_to_f64(activation)?;
                        color_transform.write(activation.context.gc_context).$field = value;
                    }
                }
                Ok(Value::Undefined.into())
            }
        )*
    }
}

color_transform_value_accessor!(
    [red_multiplier, get_red_multiplier, set_red_multiplier],
    [green_multiplier, get_green_multiplier, set_green_multiplier],
    [blue_multiplier, get_blue_multiplier, set_blue_multiplier],
    [alpha_multiplier, get_alpha_multiplier, set_alpha_multiplier],
    [red_offset, get_red_offset, set_red_offset],
    [green_offset, get_green_offset, set_green_offset],
    [blue_offset, get_blue_offset, set_blue_offset],
    [alpha_offset, get_alpha_offset, set_alpha_offset],
);

fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let formatted = format!("(redMultiplier={}, greenMultiplier={}, blueMultiplier={}, alphaMultiplier={}, redOffset={}, greenOffset={}, blueOffset={}, alphaOffset={})",
            this.get("redMultiplier", activation)?.coerce_to_string(activation)?,
            this.get("greenMultiplier", activation)?.coerce_to_string(activation)?,
            this.get("blueMultiplier", activation)?.coerce_to_string(activation)?,
            this.get("alphaMultiplier", activation)?.coerce_to_string(activation)?,
            this.get("redOffset", activation)?.coerce_to_string(activation)?,
            this.get("greenOffset", activation)?.coerce_to_string(activation)?,
            this.get("blueOffset", activation)?.coerce_to_string(activation)?,
            this.get("alphaOffset", activation)?.coerce_to_string(activation)?
    );

    Ok(AvmString::new_utf8(activation.context.gc_context, formatted).into())
}

fn concat<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let [other, ..] = args {
        let (this, other) = match (
            ColorTransformObject::cast(this.into()),
            ColorTransformObject::cast(*other),
        ) {
            (Some(this), Some(other)) => (this, other.read().clone()),
            _ => return Ok(Value::Undefined),
        };

        let mut this = this.write(activation.context.gc_context);
        *this = ColorTransformObject {
            red_multiplier: other.red_multiplier * this.red_multiplier,
            green_multiplier: other.green_multiplier * this.green_multiplier,
            blue_multiplier: other.blue_multiplier * this.blue_multiplier,
            alpha_multiplier: other.alpha_multiplier * this.alpha_multiplier,
            red_offset: (other.red_offset * this.red_multiplier) + this.red_offset,
            green_offset: (other.green_offset * this.green_multiplier) + this.green_offset,
            blue_offset: (other.blue_offset * this.blue_multiplier) + this.blue_offset,
            alpha_offset: (other.alpha_offset * this.alpha_multiplier) + this.alpha_offset,
        };
    }

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    object.into()
}
