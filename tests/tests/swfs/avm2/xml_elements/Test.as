package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

var xml = new XML("<x><?instruction ?><!-- xx -->blabla<foo>foo1</foo><bar>bar2</bar></x>")
trace('elements() length: ' + xml.elements().length());

for each (var element in xml.elements()) {
  trace('elements() element toString: ' + element.toString());
}

var xml2 = new XML("<x><foo>foo</foo><foo>bar</foo><bar>bar</bar></x>")
trace('elements(foo) length: ' + xml2.elements("foo").length());
trace('elements(bar) length: ' + xml2.elements("bar").length());
trace('elements(baz) length: ' + xml2.elements("baz").length());