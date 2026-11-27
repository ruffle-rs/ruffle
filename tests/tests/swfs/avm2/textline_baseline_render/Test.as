package {
import flash.display.Shape;
import flash.display.Sprite;
import flash.geom.Rectangle;
import flash.text.engine.*;

[SWF(width="200", height="120", backgroundColor="#FFFFFF")]
public class Test extends Sprite {
    // TestFont.swf is generated with the AIR SDK, as FTE needs a DefineFont4 font:
    // fontswf -4 -a TestFont -u U+0020,U+0061-0064 -o TestFont.swf TestFont.ttf
    [Embed(source="TestFont.swf", fontName="TestFont")]
    private var TestFont:Class;

    public function Test() {
        // The glyphs should sit on the red baseline.
        var guides:Shape = new Shape();
        guides.graphics.beginFill(0xFF0000);
        guides.graphics.drawRect(0, 40, 200, 1);
        guides.graphics.drawRect(19, 0, 1, 120);
        guides.graphics.endFill();
        addChild(guides);

        var line:TextLine = lineAt("dca", 20);
        line.x = 20;
        line.y = 40;
        addChild(line);

        // Clipping a line to its own bounds should keep all of its glyphs.
        var clipped:TextLine = lineAt("dca", 20);
        var container:Sprite = new Sprite();
        container.x = 20;
        container.y = 90;
        container.scrollRect = clipped.getBounds(clipped);
        container.addChild(clipped);
        addChild(container);
    }

    private function lineAt(text:String, size:Number):TextLine {
        var fd:FontDescription = new FontDescription("TestFont", "normal", "normal", FontLookup.EMBEDDED_CFF);
        var block:TextBlock = new TextBlock(new TextElement(text, new ElementFormat(fd, size)));
        return block.createTextLine(null, 10000);
    }
}
}
