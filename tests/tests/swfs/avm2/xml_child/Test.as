package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

var xml: XML = new XML("<x><foo>foo1</foo><bar>bar1</bar><foo>foo2</foo></x>")
trace('child("foo") length: ' + xml.child("foo").length());
trace('child("bar") length: ' + xml.child("bar").length());
trace('child("XXXXX") length: ' + xml.child("XXX").length());

for each (var child in xml.child("foo")) {
  trace('child("foo") toString: '  + child.toString());
}
for each (var child in xml.child("bar")) {
  trace('child("bar") toString: '  + child.toString());
}

var nested: XML = new XML("<x><a b='c'><b>bbb</b></a></x>")
trace('child("a").length: ' + nested.child("a").length());
trace('child("b").length: ' + nested.child("b").length());

for each (var child in nested.child("a")) {
  trace('child("a").@b: '  + child.@b);
}
for each (var child in nested.child("b")) {
  trace('child("b") toString: '  + child.toString());
}

var complex: XML = <xml>
  <a>
    <b>a1-b1</b><b>a1-b2</b>
  </a>
  <a>
    <b>a2-b</b>
    <c>a2-c</c>
  </a>
  <a/>
</xml>;
var xml_list: XMLList = complex.a;

trace('xml_list.child("b").length():', xml_list.child("b").length());
trace('xml_list.child("c").length():', xml_list.child("c").length());
trace('xml_list.child("unknown").length():', xml_list.child("unknown").length());

trace('xml_list.child("b"):', xml_list.child("b"));
trace('xml_list.child("c"):', xml_list.child("c"));
trace('xml_list.child("unknown"):', xml_list.child("unknown"));
