package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

var xml: XML = <a attr="1"><b>bbb</b></a>;
trace('xml.hasOwnProperty("@attr"): ' + xml.hasOwnProperty("@attr"));
trace('xml.hasOwnProperty("@unknown"): ' + xml.hasOwnProperty("@unknown"));
trace('xml.hasOwnProperty("b"): ' + xml.hasOwnProperty("b"));
trace('xml.hasOwnProperty("em"): ' + xml.hasOwnProperty("em"));
trace('xml.hasOwnProperty("toXMLString"): ' + xml.hasOwnProperty("toXMLString"));
trace('xml.hasOwnProperty("isPropertyEnumerable"): ' + xml.hasOwnProperty("isPropertyEnumerable"));
