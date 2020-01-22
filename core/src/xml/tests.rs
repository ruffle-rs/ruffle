//! XML tests

use crate::xml;
use crate::xml::{XMLDocument, XMLName};
use gc_arena::rootless_arena;

/// Tests very basic parsing of a single-element document.
#[test]
fn parse_single_element() {
    rootless_arena(|mc| {
        let xml = XMLDocument::new(mc);
        xml.as_node()
            .replace_with_str(mc, "<test></test>")
            .expect("Parsed document");
        let mut roots = xml
            .as_node()
            .children()
            .expect("Parsed document should be capable of having child nodes");

        let root = roots.next().expect("Parsed document should have a root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some(XMLName::from_str("test")));

        let mut root_children = root.children().unwrap();
        assert!(root_children.next().is_none());

        assert!(roots.next().is_none());
    })
}

/// Tests double-ended traversal of child nodes via DoubleEndedIterator.
#[test]
fn double_ended_children() {
    rootless_arena(|mc| {
        let xml = XMLDocument::new(mc);
        xml.as_node()
            .replace_with_str(
                mc,
                "<test></test><test2></test2><test3></test3><test4></test4><test5></test5>",
            )
            .expect("Parsed document");

        let mut roots = xml
            .as_node()
            .children()
            .expect("Parsed document should be capable of having child nodes");

        let root = roots.next().expect("Should have first root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some(XMLName::from_str("test")));

        let root = roots.next_back().expect("Should have last root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some(XMLName::from_str("test5")));

        let root = roots.next().expect("Should have next root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some(XMLName::from_str("test2")));

        let root = roots.next_back().expect("Should have second-to-last root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some(XMLName::from_str("test4")));

        let root = roots.next().expect("Should have next root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some(XMLName::from_str("test3")));

        assert!(roots.next().is_none());
        assert!(roots.next_back().is_none());
    })
}

/// Tests walking of descendent nodes via Iterator.
#[test]
#[allow(clippy::cognitive_complexity)]
fn walk() {
    rootless_arena(|mc| {
        let xml = XMLDocument::new(mc);
        xml.as_node()
            .replace_with_str(
                mc,
                "<test><test2></test2></test><test3>test</test3><test4><test5></test5></test4>",
            )
            .expect("Parsed document");

        let mut roots = xml
            .as_node()
            .walk()
            .expect("Parsed document should be capable of having child nodes");

        let root = roots.next().expect("Should have first root");
        assert!(root.stepped_in());
        assert_eq!(root.unwrap().node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.unwrap().tag_name(), Some(XMLName::from_str("test")));

        let root = roots.next().expect("Should have first root's child");
        assert!(root.stepped_in());
        assert_eq!(root.unwrap().node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.unwrap().tag_name(), Some(XMLName::from_str("test2")));

        let root = roots
            .next()
            .expect("Should have first root's child step-out");
        assert!(root.stepped_out());
        assert_eq!(root.unwrap().node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.unwrap().tag_name(), Some(XMLName::from_str("test2")));

        let root = roots.next().expect("Should have first root step-out");
        assert!(root.stepped_out());
        assert_eq!(root.unwrap().node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.unwrap().tag_name(), Some(XMLName::from_str("test")));

        let root = roots.next().expect("Should have second root");
        assert!(root.stepped_in());
        assert_eq!(root.unwrap().node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.unwrap().tag_name(), Some(XMLName::from_str("test3")));

        let root = roots
            .next()
            .expect("Should have second root's text node step-around");
        assert!(root.stepped_around());
        assert_eq!(root.unwrap().node_type(), xml::TEXT_NODE);
        assert_eq!(root.unwrap().node_value(), Some("test".to_string()));

        let root = roots.next().expect("Should have second root");
        assert!(root.stepped_out());
        assert_eq!(root.unwrap().node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.unwrap().tag_name(), Some(XMLName::from_str("test3")));

        let root = roots.next().expect("Should have last root");
        assert!(root.stepped_in());
        assert_eq!(root.unwrap().node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.unwrap().tag_name(), Some(XMLName::from_str("test4")));

        let root = roots.next().expect("Should have last root's child");
        assert!(root.stepped_in());
        assert_eq!(root.unwrap().node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.unwrap().tag_name(), Some(XMLName::from_str("test5")));

        let root = roots
            .next()
            .expect("Should have last root's child step-out");
        assert!(root.stepped_out());
        assert_eq!(root.unwrap().node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.unwrap().tag_name(), Some(XMLName::from_str("test5")));

        let root = roots.next().expect("Should have last root step-out");
        assert!(root.stepped_out());
        assert_eq!(root.unwrap().node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.unwrap().tag_name(), Some(XMLName::from_str("test4")));

        assert!(roots.next().is_none());
    })
}

/// Tests round-trip XML writing behavior.
#[test]
fn round_trip_tostring() {
    let test_string = "<test><!-- Comment -->This is a text node</test>";

    rootless_arena(|mc| {
        let xml = XMLDocument::new(mc);
        xml.as_node()
            .replace_with_str(mc, test_string)
            .expect("Parsed document");

        let result = xml
            .as_node()
            .into_string(&mut |_| true)
            .expect("Successful toString");

        assert_eq!(test_string, result);
    })
}

/// Tests filtered XML writing behavior.
#[test]
fn round_trip_filtered_tostring() {
    let test_string = "<test><!-- Comment -->This is a text node</test>";

    rootless_arena(|mc| {
        let xml = XMLDocument::new(mc);
        xml.as_node()
            .replace_with_str(mc, test_string)
            .expect("Parsed document");

        let result = xml
            .as_node()
            .into_string(&mut |node| !node.is_comment())
            .expect("Successful toString");

        assert_eq!("<test>This is a text node</test>", result);
    })
}
