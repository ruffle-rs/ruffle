//! XML tests

use crate::xml;
use crate::xml::{XMLDocument, XMLName};
use gc_arena::rootless_arena;

#[test]
fn parse_single_element() {
    rootless_arena(|mc| {
        let xml = XMLDocument::from_str(mc, "<test></test>").expect("Parsed document");
        dbg!(xml);
        let mut roots = xml.roots();

        let root = roots.next().expect("Parsed document should have a root");
        assert_eq!(root.node_type(), xml::TEXT_NODE);
        assert_eq!(root.node_value(), Some("".to_string()));

        let root = roots
            .next()
            .expect("Parsed document should have a second root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some(XMLName::from_str("test").unwrap()));

        let mut root_children = root.children().unwrap();

        let child_text = root_children
            .next()
            .expect("Single nodes should have an empty text child");
        assert_eq!(child_text.node_type(), xml::TEXT_NODE);
        assert_eq!(child_text.node_value(), Some("".to_string()));

        assert!(root_children.next().is_none());

        assert!(roots.next().is_none());
    })
}
