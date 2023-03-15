package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

// FIXME: Implement indentation.
XML.prettyPrinting = false;

var xml = <animal id="1">Cow</animal>
trace("xml.toString(): " + xml.toString());

xml = <animals>
  <animal id="1">Cow</animal>
  <animal id="2">Pig</animal>
</animals>;
trace("xml.toString(): " + xml.toString());

xml = <foo><bar a="x" b="y" c="z"/></foo>
trace("xml.toString(): " + xml.toString());

xml = <foo><bar x="a&quot;b">&gt;&amp;&lt;</bar></foo>;
trace("xml.toString(): " + xml.toString());
