use crate::avm2::error::type_error;
use crate::avm2::{Activation, ArrayObject, Error, Object, TObject, Value};
use ruffle_render::filters::{
    BevelFilter, BevelFilterType, BlurFilter, ColorMatrixFilter, ConvolutionFilter, Filter,
};
use swf::Color;

pub trait FilterAvm2Ext {
    fn from_avm2_object<'gc>(
        activation: &mut Activation<'_, 'gc>,
        object: Object<'gc>,
    ) -> Result<Filter, Error<'gc>>;

    fn as_avm2_object<'gc>(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Object<'gc>, Error<'gc>>;
}

impl FilterAvm2Ext for Filter {
    fn from_avm2_object<'gc>(
        activation: &mut Activation<'_, 'gc>,
        object: Object<'gc>,
    ) -> Result<Filter, Error<'gc>> {
        let bevel_filter = activation.avm2().classes().bevelfilter;
        if object.is_of_type(bevel_filter, activation) {
            return BevelFilter::from_avm2_object(activation, object);
        }

        let blur_filter = activation.avm2().classes().blurfilter;
        if object.is_of_type(blur_filter, activation) {
            return BlurFilter::from_avm2_object(activation, object);
        }

        let color_matrix_filter = activation.avm2().classes().colormatrixfilter;
        if object.is_of_type(color_matrix_filter, activation) {
            return ColorMatrixFilter::from_avm2_object(activation, object);
        }

        let convolution_filter = activation.avm2().classes().convolutionfilter;
        if object.is_of_type(convolution_filter, activation) {
            return ConvolutionFilter::from_avm2_object(activation, object);
        }

        Err(Error::AvmError(type_error(
            activation,
            &format!(
                "Type Coercion failed: cannot convert {object:?} to flash.filters.BitmapFilter."
            ),
            1034,
        )?))
    }

    fn as_avm2_object<'gc>(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        match self {
            Filter::BevelFilter(filter) => filter.as_avm2_object(activation),
            Filter::BlurFilter(filter) => filter.as_avm2_object(activation),
            Filter::ColorMatrixFilter(filter) => filter.as_avm2_object(activation),
            Filter::ConvolutionFilter(filter) => filter.as_avm2_object(activation),
        }
    }
}

impl FilterAvm2Ext for BevelFilter {
    fn from_avm2_object<'gc>(
        activation: &mut Activation<'_, 'gc>,
        object: Object<'gc>,
    ) -> Result<Filter, Error<'gc>> {
        let angle = object
            .get_public_property("angle", activation)?
            .coerce_to_number(activation)?;
        let blur_x = object
            .get_public_property("blurX", activation)?
            .coerce_to_number(activation)?;
        let blur_y = object
            .get_public_property("blurY", activation)?
            .coerce_to_number(activation)?;
        let distance = object
            .get_public_property("distance", activation)?
            .coerce_to_number(activation)?;
        let highlight_alpha = object
            .get_public_property("highlightAlpha", activation)?
            .coerce_to_number(activation)?;
        let highlight_color = object
            .get_public_property("highlightColor", activation)?
            .coerce_to_u32(activation)?;
        let knockout = object
            .get_public_property("knockout", activation)?
            .coerce_to_boolean();
        let quality = object
            .get_public_property("quality", activation)?
            .coerce_to_u32(activation)?;
        let shadow_alpha = object
            .get_public_property("shadowAlpha", activation)?
            .coerce_to_number(activation)?;
        let shadow_color = object
            .get_public_property("shadowColor", activation)?
            .coerce_to_u32(activation)?;
        let strength = object
            .get_public_property("strength", activation)?
            .coerce_to_u32(activation)?;
        let bevel_type = object
            .get_public_property("type", activation)?
            .coerce_to_string(activation)?;
        Ok(Filter::BevelFilter(BevelFilter {
            shadow_color: Color::from_rgb(shadow_color, (shadow_alpha * 255.0) as u8),
            highlight_color: Color::from_rgb(highlight_color, (highlight_alpha * 255.0) as u8),
            blur_x: blur_x as f32,
            blur_y: blur_y as f32,
            angle: angle as f32,
            distance: distance as f32,
            strength: strength.clamp(0, 255) as u8,
            bevel_type: if &bevel_type == b"inner" {
                BevelFilterType::Inner
            } else if &bevel_type == b"outer" {
                BevelFilterType::Outer
            } else {
                BevelFilterType::Full
            },
            knockout,
            quality: quality.clamp(1, 15) as u8,
        }))
    }

    fn as_avm2_object<'gc>(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        activation.avm2().classes().bevelfilter.construct(
            activation,
            &[
                self.distance.into(),
                self.angle.into(),
                self.highlight_color.to_rgb().into(),
                (f64::from(self.highlight_color.a) / 255.0).into(),
                self.shadow_color.to_rgb().into(),
                (f64::from(self.shadow_color.a) / 255.0).into(),
                self.blur_x.into(),
                self.blur_y.into(),
                self.strength.into(),
                self.quality.into(),
                match self.bevel_type {
                    BevelFilterType::Inner => "inner",
                    BevelFilterType::Outer => "outer",
                    BevelFilterType::Full => "full",
                }
                .into(),
                self.knockout.into(),
            ],
        )
    }
}

impl FilterAvm2Ext for BlurFilter {
    fn from_avm2_object<'gc>(
        activation: &mut Activation<'_, 'gc>,
        object: Object<'gc>,
    ) -> Result<Filter, Error<'gc>> {
        let blur_x = object
            .get_public_property("blurX", activation)?
            .coerce_to_number(activation)?;
        let blur_y = object
            .get_public_property("blurY", activation)?
            .coerce_to_number(activation)?;
        let quality = object
            .get_public_property("quality", activation)?
            .coerce_to_u32(activation)?;
        Ok(Filter::BlurFilter(BlurFilter {
            blur_x: blur_x as f32,
            blur_y: blur_y as f32,
            quality: quality.clamp(1, 15) as u8,
        }))
    }

    fn as_avm2_object<'gc>(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        activation.avm2().classes().blurfilter.construct(
            activation,
            &[self.blur_x.into(), self.blur_y.into(), self.quality.into()],
        )
    }
}

impl FilterAvm2Ext for ColorMatrixFilter {
    fn from_avm2_object<'gc>(
        activation: &mut Activation<'_, 'gc>,
        object: Object<'gc>,
    ) -> Result<Filter, Error<'gc>> {
        let mut matrix = [0.0; 20];
        if let Some(matrix_object) = object
            .get_public_property("matrix", activation)?
            .as_object()
        {
            if let Some(array) = matrix_object.as_array_storage() {
                for i in 0..matrix.len().min(array.length()) {
                    matrix[i] = array
                        .get(i)
                        .expect("Length was already checked at this point")
                        .coerce_to_number(activation)? as f32;
                }
            }
        }
        Ok(Filter::ColorMatrixFilter(ColorMatrixFilter { matrix }))
    }

    fn as_avm2_object<'gc>(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let matrix = ArrayObject::from_storage(
            activation,
            self.matrix.iter().map(|v| Value::from(*v)).collect(),
        )?;
        activation
            .avm2()
            .classes()
            .colormatrixfilter
            .construct(activation, &[matrix.into()])
    }
}

impl FilterAvm2Ext for ConvolutionFilter {
    fn from_avm2_object<'gc>(
        activation: &mut Activation<'_, 'gc>,
        object: Object<'gc>,
    ) -> Result<Filter, Error<'gc>> {
        let mut matrix = vec![];
        if let Some(matrix_object) = object
            .get_public_property("matrix", activation)?
            .as_object()
        {
            if let Some(array) = matrix_object.as_array_storage() {
                for value in array.iter() {
                    matrix.push(
                        value
                            .unwrap_or(Value::Undefined)
                            .coerce_to_number(activation)? as f32,
                    );
                }
            }
        }
        let alpha = object
            .get_public_property("alpha", activation)?
            .coerce_to_number(activation)?;
        let bias = object
            .get_public_property("bias", activation)?
            .coerce_to_number(activation)?;
        let clamp = object
            .get_public_property("clamp", activation)?
            .coerce_to_boolean();
        let color = object
            .get_public_property("color", activation)?
            .coerce_to_u32(activation)?;
        let divisor = object
            .get_public_property("divisor", activation)?
            .coerce_to_number(activation)?;
        let matrix_x = object
            .get_public_property("matrixX", activation)?
            .coerce_to_u32(activation)?;
        let matrix_y = object
            .get_public_property("matrixY", activation)?
            .coerce_to_u32(activation)?;
        let preserve_alpha = object
            .get_public_property("preserveAlpha", activation)?
            .coerce_to_boolean();
        Ok(Filter::ConvolutionFilter(ConvolutionFilter {
            bias: bias as f32,
            clamp,
            default_color: Color::from_rgb(color, (alpha * 255.0) as u8),
            divisor: divisor as f32,
            matrix,
            matrix_x: matrix_x.clamp(0, 255) as u8,
            matrix_y: matrix_y.clamp(0, 255) as u8,
            preserve_alpha,
        }))
    }

    fn as_avm2_object<'gc>(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let matrix = ArrayObject::from_storage(
            activation,
            self.matrix.iter().map(|v| Value::from(*v)).collect(),
        )?;
        activation.avm2().classes().convolutionfilter.construct(
            activation,
            &[
                self.matrix_x.into(),
                self.matrix_y.into(),
                matrix.into(),
                self.divisor.into(),
                self.bias.into(),
                self.preserve_alpha.into(),
                self.clamp.into(),
                self.default_color.to_rgb().into(),
                (f64::from(self.default_color.a) / 255.0).into(),
            ],
        )
    }
}
