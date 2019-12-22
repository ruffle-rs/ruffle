//! XML tests

use crate::xml;
use crate::xml::{XMLDocument, XMLName};
use gc_arena::rootless_arena;

#[test]
fn parse_single_element() {
    rootless_arena(|mc| {
        let xml = XMLDocument::from_str(mc, "<test></test>").expect("Parsed document");
        dbg!(xml);
        let mut roots = xml
            .as_node()
            .children()
            .expect("Parsed document should be capable of having child nodes");

        let root = roots.next().expect("Parsed document should have a root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some(XMLName::from_str("test").unwrap()));

        let mut root_children = root.children().unwrap();
        assert!(root_children.next().is_none());

        assert!(roots.next().is_none());
    })
}
