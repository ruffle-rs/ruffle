package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

function describe(s:String):String {
  var codes:Array = [];
  for (var i:int = 0; i < s.length; i++) {
    codes.push(s.charCodeAt(i));
  }
  return "length " + s.length + ", codes " + codes.join(",");
}

for each (var ref:String in ["&#0;", "&#00;", "&#000;", "&#x0;", "&#x00;"]) {
  var text:XML = new XML("<data>" + ref + "A</data>");
  trace("text " + ref + "A: " + describe(text.children()[0].toString()));

  var attr:XML = new XML("<data label=\"" + ref + "A\"></data>");
  trace("attribute " + ref + "A: " + describe(String(attr.@label)));
}

var middle:XML = new XML("<data>A&#0;B</data>");
trace("text A&#0;B: " + describe(middle.children()[0].toString()));

var attrRoundTrip:XML = new XML("<data label=\"A&#0;B\"></data>");
trace("attribute round trip: " + attrRoundTrip.toXMLString());

trace("text round trip: " + escape(middle.toXMLString()));
