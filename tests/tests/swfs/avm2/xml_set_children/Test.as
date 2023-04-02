package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

XML.prettyPrinting = false;

var xml = <a></a>;
trace("before: " + xml.toXMLString());
xml.b = "abc";
trace("after: " + xml.toXMLString());

var xml = <a><b></b></a>;
trace("before: " + xml.toXMLString());
xml.b = "abc";
trace("after: " + xml.toXMLString());

var xml = <a><b>foobar</b></a>;
trace("before: " + xml.toXMLString());
xml.b = "abc";
trace("after: " + xml.toXMLString());

var xml = <a><b><x1>x1</x1><x2>x2</x2></b></a>;
trace("before: " + xml.toXMLString());
xml.b = "abc";
trace("after: " + xml.toXMLString());

var xml = <a><b x="1"></b></a>;
trace("before: " + xml.toXMLString());
xml.b = "abc";
trace("after: " + xml.toXMLString());

var xml = <a>xxx<foo>yyy</foo>zzz</a>;
trace("before: " + xml.toXMLString());
xml.b = "abc";
trace("after: " + xml.toXMLString());
