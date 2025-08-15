package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

var xml = <root xmlns="http://example.com" xmlns:ns1="http://xxx.com/">
  <ns1:foo xmlns:ns2="http://yyy.com/">
    <ns2:bar abc="1" ns1:def="2" ns2:ghi="3">bar</ns2:bar>
  </ns1:foo>
</root>;

trace("// xml.toXMLString()");
trace(xml.toXMLString());

trace("// xml.child(0).toXMLString()");
trace(xml.child(0).toXMLString());

trace("// xml.child(0).child(0).toXMLString()");
trace(xml.child(0).child(0).toXMLString());

// TODO: Flash invents a prefix ...
// trace("// xml.child(0).child(0).copy().toXMLString()");
// trace(xml.child(0).child(0).copy().toXMLString());
