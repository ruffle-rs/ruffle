package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

import flash.xml.XMLDocument;
import flash.xml.XMLNode;

var documents = [
  new XMLDocument('<root/>'),
  new XMLDocument('<root xmlns="http://example.org"><foo hello="world" /></root>'),
  new XMLDocument('<root xmlns="http://example.org" xmlns:ns1="http://ns1.invalid"><ns1:foo hello="world"><bar ns1:attr="hey" /></ns1:foo></root>'),
  new XMLDocument('<ns1:root xmlns:ns1="http://ns1.invalid" />'),
  // TODO
  // new XMLDocument('<xml:root xml:foo="bar" />'),
];

function dump(node: XMLNode): * {
  trace("// node.toString()");
  trace(node.toString());
  trace("// node.prefix");
  trace(node.prefix);
  trace("// node.localName");
  trace(node.localName);
  trace("// node.namespaceURI");
  trace(node.namespaceURI);
  trace("// attributes");
  for (var attr in node.attributes) {
    trace(attr + " = " + node.attributes[attr]);
  }
  trace("// node.getNamespaceForPrefix('ns1')")
  trace(node.getNamespaceForPrefix('ns1'));
  trace("// node.getPrefixForNamespace('http://ns1.invalid')")
  trace(node.getPrefixForNamespace('http://ns1.invalid'));

  for each (var child in node.childNodes) {
    dump(child);
  }
}

for each (var doc in documents) {
  trace("\n// doc.toString()");
  trace(doc.toString());

  dump(doc.childNodes[0]);
}