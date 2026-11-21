package {
import flash.display.Sprite;
import flash.geom.Rectangle;
import flash.text.engine.*;

public class Test extends Sprite {
    // TestFont.swf is generated with the AIR SDK, as FTE needs a DefineFont4 font:
    // fontswf -4 -a TestFont -u U+0020,U+0061-0064 -o TestFont.swf TestFont.ttf
    [Embed(source="TestFont.swf", fontName="TestFont")]
    private var TestFont:Class;

    public function Test() {
        for each (var text:String in ["a", "a d", "dca", "a\n", "ad\r", "da\u2029", "ad\r\n"]) {
            for each (var size:Number in [10, 20]) {
                var line:TextLine = lineAt(text, size);
                trace(escape(text) + " size=" + size + " atomCount=" + line.atomCount
                    + " width=" + line.width + " height=" + line.height
                    + " bounds=" + rect(line.getBounds(line)));
                for (var i:int = 0; i < line.atomCount; i++) {
                    trace("  atom " + i + ": " + rect(line.getAtomBounds(i)));
                }
            }
        }

        var dca:TextLine = lineAt("dca", 20);
        for each (var index:int in [-1, 3, 100]) {
            try {
                trace("atom " + index + ": " + rect(dca.getAtomBounds(index)));
            } catch (e:Error) {
                trace("atom " + index + ": " + e.name + " #" + e.errorID);
            }
        }

        var container:Sprite = new Sprite();
        addChild(container);
        dca.x = 10;
        dca.y = 30;
        container.addChild(dca);
        trace("in container: " + rect(dca.getBounds(container)) + " container=" + rect(container.getBounds(container)));
    }

    private static function rect(r:Rectangle):String {
        return "(x=" + r.x + ", y=" + r.y + ", w=" + r.width + ", h=" + r.height + ")";
    }

    private function lineAt(text:String, size:Number):TextLine {
        var fd:FontDescription = new FontDescription("TestFont", "normal", "normal", FontLookup.EMBEDDED_CFF);
        var block:TextBlock = new TextBlock(new TextElement(text, new ElementFormat(fd, size)));
        return block.createTextLine(null, 10000);
    }
}
}
