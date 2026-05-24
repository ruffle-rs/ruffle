package {
import flash.display.Sprite;
import flash.events.EventDispatcher;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        var line:TextLine = new TextBlock(
            new TextElement("mirror", new ElementFormat(fd, 20))
        ).createTextLine(null, 1000);

        try {
            trace("mirror length: " + line.mirrorRegions.length);
        } catch (e:Error) {
            trace("mirror length error: " + e.errorID);
        }

        try {
            trace("mirror lookup null: " + (line.getMirrorRegion(new EventDispatcher()) == null));
        } catch (e:Error) {
            trace("mirror lookup error: " + e.errorID);
        }

        try {
            var dump:String = line.dump();
            trace("dump shape: " + (dump.indexOf("<TextLine atomCount=6 textWidth=") == 0));
        } catch (e:Error) {
            trace("dump error: " + e.errorID);
        }
    }
}
}
