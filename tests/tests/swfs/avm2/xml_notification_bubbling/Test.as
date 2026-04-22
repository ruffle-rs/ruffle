package {
import flash.display.Sprite;

public class Test extends Sprite {}
}

function notifier(currentTarget:Object, command:String, target:Object,
                    value:Object, detail:Object):void {
    trace("# notifier")
    trace("currentTarget = " + currentTarget.toXMLString());
    trace("command = " + command);
    trace("target = " + target.toXMLString());
    trace("value = " + value);
    trace("detail = " + detail);
    trace("");
}

function testit(xml: XML):void {
    xml.first.second.@a = "added a";
    xml.first.second.@b = "added b";
    xml.first.@c = "added c";
    xml.@d = "added d";

    xml.first.second.@a = "changed a";
    xml.first.second.@b = "changed b";
    xml.first.@c = "changed c";
    xml.@d = "changed d";
}

var xml1:XML = <root><first><second></second></first></root>;
xml1.setNotification(notifier);
testit(xml1);

var xml2:XML = <root2><first><second></second></first></root2>;
xml2.setNotification(notifier);
xml2.first[0].setNotification(notifier);
xml2.first.second[0].setNotification(notifier);
testit(xml2);
