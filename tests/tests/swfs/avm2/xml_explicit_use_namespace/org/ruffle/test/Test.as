package org.ruffle.test {
    import flash.display.Sprite;

    import org.ruffle.namespaces.*;

    use namespace example;

    public class Test extends Sprite {
        public var xml: XML = <root xmlns="http://example.org/"><hello>world</hello></root>

        public function Test() {
            trace(xml.toXMLString());
            trace("xml.hello: " + xml.hello);
            trace('xml["hello"]: ' + xml["hello"]);
        }
    }
}
