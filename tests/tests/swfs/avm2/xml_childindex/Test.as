package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

var xml = <xml>Test<a attr="123">a</a><b><x/>b</b></xml>

trace("xml.childIndex():", xml.childIndex());
trace("xml.children()[0].childIndex():", xml.children()[0].childIndex());
trace("xml.a.childIndex():", xml.a.childIndex());
trace("xml.b.childIndex():", xml.b.childIndex());
trace("xml.b.x.childIndex():", xml.b.x.childIndex());
trace("xml.b.children()[1].childIndex()", xml.b.children()[1].childIndex());
trace("xml.a.@attr.childIndex()", xml.a.@attr.childIndex());
