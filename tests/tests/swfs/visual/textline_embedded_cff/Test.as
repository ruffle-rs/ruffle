package {
import flash.display.Sprite;
import flash.text.Font;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.FontLookup;
import flash.text.engine.FontPosture;
import flash.text.engine.FontWeight;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="FteCffVisual", embedAsCFF="true", unicodeRange="U+0020-U+007A")]
    private var FteCffVisualFont:Class;

    public function Test() {
        Font.registerFont(FteCffVisualFont);

        graphics.beginFill(0xffffff);
        graphics.drawRect(0, 0, 240, 120);
        graphics.endFill();

        graphics.lineStyle(1, 0xff0000);
        graphics.drawRect(9, 37, 151, 34);

        var fd:FontDescription = new FontDescription("FteCffVisual", FontWeight.NORMAL, FontPosture.NORMAL, FontLookup.EMBEDDED_CFF);
        var ef:ElementFormat = new ElementFormat(fd, 30, 0x000000);
        var tb:TextBlock = new TextBlock(new TextElement("iiiiii WWW", ef));
        var line:TextLine = tb.createTextLine(null, 1000);
        line.x = 12;
        line.y = 64;
        addChild(line);
    }
}
}
