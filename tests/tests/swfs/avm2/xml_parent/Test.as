package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

XML.prettyPrinting = false;
var xml = new XML("<x><foo foo='foo'><bar><baz>baz1</baz></bar></foo></x>");
var foo = xml.foo;
var bar = foo.bar;
var baz = bar.baz;

trace("// xml.parent()");
trace(xml.parent());

trace("// foo.parent()");
trace(foo.parent());

trace("// bar.parent()");
trace(bar.parent());

trace("// baz.parent()");
trace(baz.parent());