package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

var xml = new XML("<x><foo>foo1</foo><bar>bar1</bar><foo>foo2</foo></x>")
trace('child("foo") length: ' + xml.child("foo").length());
trace('child("bar") length: ' + xml.child("bar").length());
trace('child("XXXXX") length: ' + xml.child("XXX").length());

for each (var child in xml.child("foo")) {
  trace('child("foo") toString: '  + child.toString());
}
for each (var child in xml.child("bar")) {
  trace('child("bar") toString: '  + child.toString());
}

var nested = new XML("<x><a b='c'><b>bbb</b></a></x>")
trace('child("a").length: ' + nested.child("a").length());
trace('child("b").length: ' + nested.child("b").length());

for each (var child in nested.child("a")) {
  trace('child("a").@b: '  + child.@b);
}
for each (var child in nested.child("b")) {
  trace('child("b") toString: '  + child.toString());
}
