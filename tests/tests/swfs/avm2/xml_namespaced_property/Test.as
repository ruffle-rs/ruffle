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

var example = new Namespace("http://example.org/");
trace("xml.foo.toString()", xml.foo.toString());
trace("xml.example::foo.toString()", xml.example::foo.toString());
trace("xml.hello.toString()", xml.hello.toString());
trace("xml.example::hello.toString()", xml.example::hello.toString());

xml.example::test = "abc";
trace("xml.example::test", xml.example::test);
