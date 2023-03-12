package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

var xml = new XML("<x><?instruction ?><!-- xx -->blabla<foo>foo1</foo><bar>bar2</bar></x>")
trace('elements() length: ' + xml.elements().length());

for each (var element in xml.elements()) {
  trace('elements() element toString: ' + element.toString());
}
