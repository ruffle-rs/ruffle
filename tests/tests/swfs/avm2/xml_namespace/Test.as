package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

var xml = <foo>bar</foo>;
trace("xml.namespace(): " + xml.namespace());

var ns = xml.namespace();
trace("ns.prefix: " + ns.prefix);
trace("ns.uri: " + ns.uri);
