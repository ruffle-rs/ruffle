//! flash.geom.ColorTransform object

use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Activation, Error, Object, Value};
use crate::string::AvmString;
use gc_arena::{Collect, Gc};
use ruffle_macros::istr;
use std::cell::Cell;
use swf::{ColorTransform, Fixed8};

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct ColorTransformObject {
    red_multiplier: Cell<f64>,
    green_multiplier: Cell<f64>,
    blue_multiplier: Cell<f64>,
    alpha_multiplier: Cell<f64>,
    red_offset: Cell<f64>,
    green_offset: Cell<f64>,
    blue_offset: Cell<f64>,
    alpha_offset: Cell<f64>,
}

impl<'gc> ColorTransformObject {
    pub const fn identity() -> Self {
        Self {
            red_multiplier: Cell::new(1.0),
            green_multiplier: Cell::new(1.0),
            blue_multiplier: Cell::new(1.0),
            alpha_multiplier: Cell::new(1.0),
            red_offset: Cell::new(0.0),
            green_offset: Cell::new(0.0),
            blue_offset: Cell::new(0.0),
            alpha_offset: Cell::new(0.0),
        }
    }

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
        let constructor = activation.prototypes().color_transform_constructor;
        constructor.construct(activation, &args)
    }

    pub fn cast(value: Value<'gc>) -> Option<Gc<'gc, Self>> {
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
            r_multiply: Fixed8::from_f64(object.red_multiplier.get()),
            g_multiply: Fixed8::from_f64(object.green_multiplier.get()),
            b_multiply: Fixed8::from_f64(object.blue_multiplier.get()),
            a_multiply: Fixed8::from_f64(object.alpha_multiplier.get()),
            r_add: object.red_offset.get() as i16,
            g_add: object.green_offset.get() as i16,
            b_add: object.blue_offset.get() as i16,
            a_add: object.alpha_offset.get() as i16,
        }
    }
}

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
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

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.native_class(constructor, None, super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    class
}

fn constructor<'gc>(
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
                red_multiplier: Cell::new(red_multiplier),
                green_multiplier: Cell::new(green_multiplier),
                blue_multiplier: Cell::new(blue_multiplier),
                alpha_multiplier: Cell::new(alpha_multiplier),
                red_offset: Cell::new(red_offset),
                green_offset: Cell::new(green_offset),
                blue_offset: Cell::new(blue_offset),
                alpha_offset: Cell::new(alpha_offset),
            }
        }
        [color_transform] => {
            if let Some(color_transform) = ColorTransformObject::cast(*color_transform) {
                (*color_transform).clone()
            } else {
                ColorTransformObject::identity()
            }
        }
        _ => ColorTransformObject::identity(),
    };
    this.set_native(
        activation.gc(),
        NativeObject::ColorTransform(Gc::new(activation.gc(), color_transform)),
    );
    Ok(this.into())
}

fn get_rgb<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(color_transform) = ColorTransformObject::cast(this.into()) {
        let rgb = ((color_transform.red_offset.get() as i32) << 16)
            | ((color_transform.green_offset.get() as i32) << 8)
            | (color_transform.blue_offset.get() as i32);
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
            color_transform.red_multiplier.set(0.0);
            color_transform.green_multiplier.set(0.0);
            color_transform.blue_multiplier.set(0.0);
            color_transform.red_offset.set(r.into());
            color_transform.green_offset.set(g.into());
            color_transform.blue_offset.set(b.into());
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
                    Ok(color_transform.$field.get().into())
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
                        color_transform.$field.set(value);
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
            this.get(istr!("redMultiplier"), activation)?.coerce_to_string(activation)?,
            this.get(istr!("greenMultiplier"), activation)?.coerce_to_string(activation)?,
            this.get(istr!("blueMultiplier"), activation)?.coerce_to_string(activation)?,
            this.get(istr!("alphaMultiplier"), activation)?.coerce_to_string(activation)?,
            this.get(istr!("redOffset"), activation)?.coerce_to_string(activation)?,
            this.get(istr!("greenOffset"), activation)?.coerce_to_string(activation)?,
            this.get(istr!("blueOffset"), activation)?.coerce_to_string(activation)?,
            this.get(istr!("alphaOffset"), activation)?.coerce_to_string(activation)?
    );

    Ok(AvmString::new_utf8(activation.gc(), formatted).into())
}

fn concat<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let [other, ..] = args {
        let (this, other) = match (
            ColorTransformObject::cast(this.into()),
            ColorTransformObject::cast(*other),
        ) {
            (Some(this), Some(other)) => (this, other),
            _ => return Ok(Value::Undefined),
        };

        // The offsets are set before the multipliers because the calculations
        // for the offsets depend on the original value of the multipliers.
        this.red_offset
            .set((other.red_offset.get() * this.red_multiplier.get()) + this.red_offset.get());
        this.green_offset.set(
            (other.green_offset.get() * this.green_multiplier.get()) + this.green_offset.get(),
        );
        this.blue_offset
            .set((other.blue_offset.get() * this.blue_multiplier.get()) + this.blue_offset.get());
        this.alpha_offset.set(
            (other.alpha_offset.get() * this.alpha_multiplier.get()) + this.alpha_offset.get(),
        );

        // Now we can set the multipliers.
        this.red_multiplier
            .set(other.red_multiplier.get() * this.red_multiplier.get());
        this.green_multiplier
            .set(other.green_multiplier.get() * this.green_multiplier.get());
        this.blue_multiplier
            .set(other.blue_multiplier.get() * this.blue_multiplier.get());
        this.alpha_multiplier
            .set(other.alpha_multiplier.get() * this.alpha_multiplier.get());
    }

    Ok(Value::Undefined)
}
