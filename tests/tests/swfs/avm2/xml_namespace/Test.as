package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

testXML(<root />);
testXML(<root xmlns="http://example.org" />);
testXML(<root xmlns:other="http://other.com" />);
testXML(<root xmlns:other="http://other.com" xmlns="http://example.org" />);

function testXML(xml: XML) {
  trace(xml.toXMLString());
  trace("// namespace()");
  dumpNS(xml.namespace());
  trace('// namespace("")');
  dumpNS(xml.namespace(""));
  trace('// namespace("other")');
  dumpNS(xml.namespace("other"));
  trace("");
}

function dumpNS(ns: Namespace) {
  if (ns) {
    trace("ns.prefix: " + ns.prefix);
    trace("ns.uri: " + ns.uri);
  } else {
    trace("ns is null");
  }
}
