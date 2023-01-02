//! XMLList builtin and prototype

pub use crate::avm2::object::xml_list_allocator;
use crate::avm2::{
    e4x::{simple_content_to_string, E4XNode, E4XNodeKind},
    object::E4XOrXml,
    Activation, Error, Object, TObject, Value,
};

fn has_simple_content_inner(children: &[E4XOrXml<'_>]) -> bool {
    match children {
        [] => true,
        [child] => child.node().has_simple_content(),
        _ => children
            .iter()
            .all(|child| !matches!(&*child.node().kind(), E4XNodeKind::Element { .. })),
    }
}

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.unwrap().as_xml_list().unwrap();
    let value = args[0];

    match E4XNode::parse(value, activation) {
        Ok(nodes) => {
            this.set_children(
                activation.context.gc_context,
                nodes.into_iter().map(E4XOrXml::E4X).collect(),
            );
        }
        Err(e) => {
            return Err(Error::RustError(
                format!("Failed to parse XML: {e:?}").into(),
            ))
        }
    }

    Ok(Value::Undefined)
}

pub fn has_simple_content<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.unwrap().as_xml_list().unwrap();
    let children = list.children();
    Ok(has_simple_content_inner(&children).into())
}

pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.unwrap().as_xml_list().unwrap();
    let children = list.children();
    if has_simple_content_inner(&children) {
        Ok(simple_content_to_string(children.iter().cloned(), activation)?.into())
    } else {
        Err("XMLList.toString() for non-simple content: not yet implemented".into())
    }
}

pub fn length<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.unwrap().as_xml_list().unwrap();
    let children = list.children();
    Ok(children.len().into())
}
