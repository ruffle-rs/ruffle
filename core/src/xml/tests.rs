//! XML tests

use crate::string::WStr;
use crate::xml;
use crate::xml::XmlDocument;
use gc_arena::rootless_arena;

/// Tests very basic parsing of a single-element document.
#[test]
fn parse_single_element() {
    rootless_arena(|mc| {
        let mut xml = XmlDocument::new(mc);
        xml.replace_with_str(mc, WStr::from_units(b"<test></test>"), true, false)
            .expect("Parsed document");
        let mut roots = xml.as_node().children();

        let root = roots.next().expect("Parsed document should have a root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some("test".into()));

        let mut root_children = root.children();
        assert!(root_children.next().is_none());

        assert!(roots.next().is_none());
    })
}

/// Tests double-ended traversal of child nodes via DoubleEndedIterator.
#[test]
fn double_ended_children() {
    rootless_arena(|mc| {
        let mut xml = XmlDocument::new(mc);
        xml.replace_with_str(
            mc,
            WStr::from_units(
                b"<test></test><test2></test2><test3></test3><test4></test4><test5></test5>",
            ),
            true,
            false,
        )
        .expect("Parsed document");

        let mut roots = xml.as_node().children();

        let root = roots.next().expect("Should have first root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some("test".into()));

        let root = roots.next_back().expect("Should have last root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some("test5".into()));

        let root = roots.next().expect("Should have next root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some("test2".into()));

        let root = roots.next_back().expect("Should have second-to-last root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some("test4".into()));

        let root = roots.next().expect("Should have next root");
        assert_eq!(root.node_type(), xml::ELEMENT_NODE);
        assert_eq!(root.tag_name(), Some("test3".into()));

        assert!(roots.next().is_none());
        assert!(roots.next_back().is_none());
    })
}

/// Tests round-trip XML writing behavior.
#[test]
fn round_trip_tostring() {
    let test_string = b"<test><!-- Comment -->This is a text node</test>";

    rootless_arena(|mc| {
        let mut xml = XmlDocument::new(mc);
        xml.replace_with_str(mc, WStr::from_units(test_string), true, false)
            .expect("Parsed document");

        let result = xml
            .as_node()
            .into_string(&|_| true)
            .expect("Successful toString");

        assert_eq!(std::str::from_utf8(test_string).unwrap(), result);
    })
}

/// Tests filtered XML writing behavior.
#[test]
fn round_trip_filtered_tostring() {
    let test_string = b"<test><!-- Comment -->This is a text node</test>";

    rootless_arena(|mc| {
        let mut xml = XmlDocument::new(mc);
        xml.replace_with_str(mc, WStr::from_units(test_string), true, false)
            .expect("Parsed document");

        let result = xml
            .as_node()
            .into_string(&|node| !node.is_comment())
            .expect("Successful toString");

        assert_eq!("<test>This is a text node</test>", result);
    })
}

/// Tests ignoring whitespace nodes.
#[test]
fn ignore_white() {
    rootless_arena(|mc| {
        let mut xml = XmlDocument::new(mc);
        xml.replace_with_str(
            mc,
            WStr::from_units(b"<test>   <test2>   <test3> foo </test3>   </test2>   </test>"),
            true,
            true,
        )
        .expect("Parsed document");

        let mut root = xml.as_node().children();

        let mut node = root.next().expect("Should have root");
        assert_eq!(node.node_type(), xml::ELEMENT_NODE);
        assert_eq!(node.tag_name(), Some("test".into()));

        node = node.children().next().expect("Should have children");
        assert_eq!(node.node_type(), xml::ELEMENT_NODE);
        assert_eq!(node.tag_name(), Some("test2".into()));

        node = node.children().next().expect("Should have children");
        assert_eq!(node.node_type(), xml::ELEMENT_NODE);
        assert_eq!(node.tag_name(), Some("test3".into()));

        node = node.children().next().expect("Should have text");
        assert_eq!(node.node_type(), xml::TEXT_NODE);
        assert_eq!(node.node_value(), Some(" foo ".into()));

        assert!(root.next().is_none());
    })
}
