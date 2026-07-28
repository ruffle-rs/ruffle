package {
import flash.display.Sprite;

public class Test extends Sprite {
    public function Test() {
        var xml1:XML = <root><a/><b/></root>;
        var children:XMLList = xml1.children();
        var a:XML = children[0];

        delete children[0];

        trace("a.toXMLString(): " + a.toXMLString());
        trace("a.parent(): " + a.parent());
        trace("a.childIndex(): " + a.childIndex());

        var xml2:XML = <root attr="1"/>;
        var attributes:XMLList = xml2.attributes();
        var attr:XML = attributes[0];

        delete attributes[0];

        trace("attr.toXMLString(): " + attr.toXMLString());
        trace("atrr.parent(): " + attr.parent());
        trace("atrr.childIndex(): " + attr.childIndex());
    }
}

}
