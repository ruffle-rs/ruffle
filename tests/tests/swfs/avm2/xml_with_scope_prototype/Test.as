// This test verifies that XML prototype methods (name, localName, nodeKind,
// toString, children) are found via with-scope resolution when looked up
// using public namespace QNames.
//
// In real Flash, the mxmlc compiler generates QName(PackageNamespace(""), "name")
// for method calls inside `with(xmlNode) { ... }` blocks. The XML vtable only
// has these methods in the AS3 namespace, but they are also defined on
// XML.prototype in the public namespace (see XML.as lines 183+).
//
// The test SWF was generated using hand-crafted ABC bytecode (see gen_swf.rs)
// because the key requirement is that findpropstrict uses a PUBLIC namespace
// QName (not AS3), which is what mxmlc generates for with-scope code.
//
// Pseudocode equivalent:
//
//   var xml:XML = new XML("<root><item>hello</item><item>world</item></root>");
//   with (xml) {
//       trace("name() via with-scope: " + name());
//       trace("localName() via with-scope: " + localName());
//       trace("nodeKind() via with-scope: " + nodeKind());
//       trace("toString() via with-scope: " + toString());
//       trace("children().length() via with-scope: " + children().length());
//   }
//
// Without the fix, findpropstrict fails to find these methods on the XML
// with-scope because:
// 1. vtable traits only match AS3 namespace (not public)
// 2. has_own_property only checks E4X child elements
// 3. prototype chain was not checked (the bug)
