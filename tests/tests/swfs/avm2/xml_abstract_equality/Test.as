package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

XML.prettyPrinting = false;

var simple: XML = <a>abc</a>;
trace("simple:", simple.toXMLString());
trace("simple == simple:", simple == simple);
trace('simple == "abc":', simple == "abc");
trace("simple == simple.children()[0] [text]:", simple == simple.children()[0]);
trace("simple == <a>abc</a>", simple == <a>abc</a>);
trace('simple == <a hello="world">abc</a>', simple == <a hello="world">abc</a>);
trace("simple == <xxx>abc</xxx>", simple == <xxx>abc</xxx>);

var true_XML: XML = <a>true</a>;
trace("true_XML:", true_XML.toXMLString());
trace("true_XML == true:", true_XML == true);

var attr: XML = <a hello="world" />;
trace("attr:", attr.toXMLString());
trace("attr.@hello == attr.@hello:", attr.@hello == attr.@hello);
trace('attr.@hello == "world":', attr.@hello == "world");
trace("attr.@hello == <x>world</x>:", attr.@hello == <x>world</x>);
trace('attr.@hello == "foobar":', attr.@hello == "foobar");
trace("attr.@hello == <x><y>world</y></x>:", attr.@hello == <x><y>world</y></x>);

var xml: XML = <x><a>a</a><t>true</t><n>123</n><b>b1</b><b>b2</b></x>;
trace("xml:", xml.toXMLString());
trace("xml == xml:", xml == xml);
var xml_a: XMLList = xml.a;
trace("xml_a == xml_a:", xml_a == xml_a);
trace("xml.a == xml.a:", xml.a == xml.a);
trace('xml.a == "a":', xml.a == "a");
trace('xml.t == true:', xml.t == true);
trace('xml.n == 123:', xml.n == 123);
trace('xml.n == "123":', xml.n == "123");
trace('xml.n == 42:', xml.n == 42);
trace("xml.b == xml.b:", xml.b == xml.b);
trace('xml.b == xml.a:', xml.b == xml.a);

var other: XML = <xxx><y>b1</y><y>b2</y></xxx>;
trace("other:", other.toXMLString());
trace("xml.b == other.y:", xml.b == other.y);

var other2: XML = <xxx><b>b1</b><b>b2</b></xxx>
trace("other2:", other2.toXMLString());
trace("xml.b == other2.b:", xml.b == other2.b)

var attrs: XML = <x a="b1" b="b2" />;
trace("attrs:", attrs.toXMLString());
trace('attrs == <x a="x1" b="x2" />', attrs == <x a="x1" b="x2" />);
trace('attrs == <x b="b2" a="b1" />', attrs == <x b="b2" a="b1" />);
trace("xml.b == attrs.attributes():", xml.b == attrs.attributes());

trace('xml.child("unknown") == undefined:', xml.child("unknown") == undefined);
trace('xml.child("unknown") == "":', xml.child("unknown") == "");
