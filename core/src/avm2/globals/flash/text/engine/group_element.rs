use crate::avm2::activation::Activation;
use crate::avm2::error::{Error, Error2004Type, Error2006Type, make_error_2004, make_error_2006};
use crate::avm2::object::{ContentElementObject, ElementData, Object, VectorObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_content_element_object()
        .unwrap();

    let mut data = this.element_data_mut(activation.gc());
    *data = ElementData::Group {
        elements: Vec::new(),
    };

    Ok(Value::Undefined)
}

pub fn get_element_count<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_content_element_object()
        .unwrap();

    let data = this.element_data();
    let ElementData::Group { elements } = &*data else {
        unreachable!("Data can only have been set to Group");
    };

    Ok(Value::from_usize_lossy(elements.len()))
}

pub fn get_element_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_content_element_object()
        .unwrap();

    let data = this.element_data();
    let ElementData::Group { elements } = &*data else {
        unreachable!("Data can only have been set to Group");
    };

    let index = args.get_i32(0);

    let index = usize::try_from(index)
        .map_err(|_| make_error_2006(activation, Error2006Type::RangeError))?;

    let element = elements
        .get(index)
        .copied()
        .ok_or_else(|| make_error_2006(activation, Error2006Type::RangeError))?;

    Ok(element.into())
}

pub fn set_elements<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_content_element_object()
        .unwrap();

    let data = this.element_data();
    let ElementData::Group { elements } = &*data else {
        unreachable!("Data can only have been set to Group");
    };

    let element_count = elements.len();

    drop(data);

    let new_elements = args
        .try_get_object(0)
        .map(|o| o.as_vector_object().unwrap());

    replace_elements_impl(activation, this, 0, element_count, new_elements)?;

    Ok(Value::Undefined)
}

pub fn replace_elements<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_content_element_object()
        .unwrap();

    let begin_index = args.get_i32(0);
    let end_index = args.get_i32(1);

    let new_elements = args
        .try_get_object(2)
        .map(|o| o.as_vector_object().unwrap());

    let begin_index = usize::try_from(begin_index)
        .map_err(|_| make_error_2006(activation, Error2006Type::RangeError))?;

    let end_index = usize::try_from(end_index)
        .map_err(|_| make_error_2006(activation, Error2006Type::RangeError))?;

    // This is some sort of special case that doesn't throw.
    if begin_index == end_index && new_elements.is_none() {
        Ok(Value::Null)
    } else {
        let result = replace_elements_impl(activation, this, begin_index, end_index, new_elements)?;

        Ok(result.into())
    }
}

fn replace_elements_impl<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: ContentElementObject<'gc>,
    begin_index: usize,
    end_index: usize,
    new_elements: Option<VectorObject<'gc>>,
) -> Result<VectorObject<'gc>, Error<'gc>> {
    let mut data = this.element_data_mut(activation.gc());
    let ElementData::Group { elements } = &mut *data else {
        unreachable!("Data can only have been set to Group");
    };

    if begin_index > elements.len() || end_index > elements.len() {
        return Err(make_error_2006(activation, Error2006Type::RangeError));
    }

    let new_elements = new_elements.map(|o| o.storage());

    let new_elements = if let Some(new_elements) = new_elements {
        // TODO perform more validation:
        // - Can't insert the same element twice
        // - Can't create a loop
        new_elements
            .iter()
            .map(|e| {
                let content_element = e.as_object().and_then(|o| o.as_content_element_object());

                let Some(content_element) = content_element else {
                    // There was a `null` value in the vector
                    return Err(make_error_2004(activation, Error2004Type::ArgumentError));
                };

                if Object::ptr_eq(content_element, this) {
                    // Can't insert a group element into itself
                    return Err(make_error_2004(activation, Error2004Type::ArgumentError));
                }

                let data = content_element.element_data();
                if matches!(&*data, ElementData::Invalid) {
                    // Can't insert an instance of a user-defined subclass of
                    // `ContentElement` into a group element
                    return Err(make_error_2004(activation, Error2004Type::ArgumentError));
                }

                Ok(content_element)
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        // Passing `null` results in no elements
        Vec::new()
    };

    // NOTE: FP can segfault here. The logic for what pattern makes it crash
    // seems to be as follows:

    // 1. Flatten the children of any descendant `GroupElement`s in the
    //    `new_elements` array, to create an array of only `TextElement`,
    //    `GraphicElement`, and empty `GroupElement`s. Iterate over the
    //    resulting array.
    // 2. If this iteration encounters an empty `GroupElement` or a `TextElement`
    //    with `null` text *anytime after* encountering a `TextElement` that has
    //    non-`null`, non-empty text, FP crashes.

    // I haven't reproduced this crash here, but it might be necessary to bail
    // out to avoid having to handle cases in other FTE code that FP cannot
    // encounter.

    // This crash affects at least both `replaceElements` and `setElements`.

    let removed_elements = elements
        .splice(begin_index..end_index, new_elements)
        .map(Value::from)
        .collect::<Vec<_>>();

    // Return the elements that were removed
    let new_vs = VectorStorage::from_values(
        removed_elements,
        false,
        Some(activation.avm2().class_defs().contentelement),
    );

    Ok(VectorObject::from_vector(new_vs, activation))
}
