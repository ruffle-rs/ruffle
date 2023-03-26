package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}


var xml = <xml xmlns:example="http://example.org/">
  <foo>abc</foo>
  <example:hello>world</example:hello>
</xml>;

trace("xml.*::foo.toString()", xml.*::foo.toString());
trace("xml.*::hello.toString()", xml.*::hello.toString());

var ns = new Namespace("http://example.org/");

// Enable these tests when namespaces are supported.
// trace("xml.ns::foo.toString()", xml.ns::foo.toString());
// trace("xml.hello.toString()", xml.hello.toString());
// xml.ns::test = "abc";

try {
  trace("xml.ns::test", xml.ns::test);
} catch (e) {
  // This should not actually error.
  trace(e.toString());
}
