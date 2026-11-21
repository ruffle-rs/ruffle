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
        for each (var width:Number in [50, 30]) {
            var fd:FontDescription = new FontDescription("TestFont", "normal", "normal", FontLookup.EMBEDDED_CFF);
            var block:TextBlock = new TextBlock(new TextElement("dca dca dca", new ElementFormat(fd, 20)));
            var previous:TextLine = null;
            for (var i:int = 0; i < 10; i++) {
                var line:TextLine = block.createTextLine(previous, width);
                if (line == null) {
                    break;
                }

                trace("width=" + width + " line " + i + ": rawTextLength=" + line.rawTextLength + " bounds=" + rect(line.getBounds(line)));
                previous = line;
            }
        }
    }

    private static function rect(r:Rectangle):String {
        return "(x=" + r.x + ", y=" + r.y + ", w=" + r.width + ", h=" + r.height + ")";
    }
}
}
