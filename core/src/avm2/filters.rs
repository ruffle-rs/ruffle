use crate::avm2::error::type_error;
use crate::avm2::{Activation, ArrayObject, Error, Object, TObject, Value};
use ruffle_render::filters::{BlurFilter, ColorMatrixFilter, Filter};

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
        let color_matrix_filter = activation.avm2().classes().colormatrixfilter;
        if object.is_of_type(color_matrix_filter, activation) {
            return ColorMatrixFilter::from_avm2_object(activation, object);
        }

        let blur_filter = activation.avm2().classes().blurfilter;
        if object.is_of_type(blur_filter, activation) {
            return BlurFilter::from_avm2_object(activation, object);
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
            Filter::BlurFilter(filter) => filter.as_avm2_object(activation),
            Filter::ColorMatrixFilter(filter) => filter.as_avm2_object(activation),
        }
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
