package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

var xml:XML = new XML("<data>A &amp; &#39; B</data>");
trace(xml.children()[0].toString());

xml = new XML("<data label=\"A &amp; &#39; B\"></data>");
trace("");
trace(xml.@label);

xml = new XML("<data>A & &thing; B</data>");
trace("");
trace(xml.children()[0].toString());

xml = new XML("<data label=\"A & &thing; B\"></data>");
trace("");
trace(xml.@label);

xml = new XML("<data>A &&thing; B</data>");
trace("");
trace(xml.children()[0].toString());

xml = new XML("<data label=\"A &&thing; B\"></data>");
trace("");
trace(xml.@label);

xml = new XML("<data>A &&&thing; B</data>");
trace("");
trace(xml.children()[0].toString());

xml = new XML("<data label=\"A &&&thing; B\"></data>");
trace("");
trace(xml.@label);

xml = new XML("<data>A &&amp; B</data>");
trace("");
trace(xml.children()[0].toString());

xml = new XML("<data label=\"A &&amp; B\"></data>");
trace("");
trace(xml.@label);

xml = new XML("<data>A &amp;&amp; B</data>");
trace("");
trace(xml.children()[0].toString());

xml = new XML("<data label=\"A &amp;&amp; B\"></data>");
trace("");
trace(xml.@label);
