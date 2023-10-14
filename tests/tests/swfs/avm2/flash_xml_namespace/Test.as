package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

import flash.xml.XMLDocument;

var doc:XMLDocument = new XMLDocument('<xml:a/><foo xml:bar="hello"/><a xmlns:example="http://example.org"><b/><example:c/></a>');

trace("/// doc.childNodes[0].toString()");
trace(doc.childNodes[0].toString());
trace("/// doc.childNodes[0].prefix");
trace(doc.childNodes[0].prefix);
trace("/// doc.childNodes[0].localName");
trace(doc.childNodes[0].localName);
trace("/// doc.childNodes[0].namespaceURI");
trace(doc.childNodes[0].namespaceURI);

trace("/// doc.childNodes[1].toString()");
trace(doc.childNodes[1].toString());
trace("/// doc.childNodes[1].prefix");
trace(doc.childNodes[1].prefix);
trace("/// doc.childNodes[1].localName");
trace(doc.childNodes[1].localName);
trace("/// doc.childNodes[1].namespaceURI");
trace(doc.childNodes[1].namespaceURI);

trace("/// doc.childNodes[2].toString()");
trace(doc.childNodes[2].toString());
trace("/// doc.childNodes[2].prefix");
trace(doc.childNodes[2].prefix);
trace("/// doc.childNodes[2].localName");
trace(doc.childNodes[2].localName);
trace("/// doc.childNodes[2].namespaceURI");
trace(doc.childNodes[2].namespaceURI);

trace("/// doc.childNodes[2].childNodes[0].toString()");
trace(doc.childNodes[2].childNodes[0].toString());
trace("/// doc.childNodes[2].childNodes[0].prefix");
trace(doc.childNodes[2].childNodes[0].prefix);
trace("/// doc.childNodes[2].childNodes[0].localName");
trace(doc.childNodes[2].childNodes[0].localName);
trace("/// doc.childNodes[2].childNodes[0].namespaceURI");
trace(doc.childNodes[2].childNodes[0].namespaceURI);

trace("/// doc.childNodes[2].childNodes[1].toString()");
trace(doc.childNodes[2].childNodes[1].toString());
trace("/// doc.childNodes[2].childNodes[1].prefix");
trace(doc.childNodes[2].childNodes[1].prefix);
trace("/// doc.childNodes[2].childNodes[1].localName");
trace(doc.childNodes[2].childNodes[1].localName);
trace("/// doc.childNodes[2].childNodes[1].namespaceURI");
trace(doc.childNodes[2].childNodes[1].namespaceURI);

for (var name in doc.childNodes[1].attributes) {
  trace("/// attribute name");
  trace(name);
}
