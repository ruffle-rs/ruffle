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
    [Embed(source="TestFont.ttf", fontName="FteCffTest", embedAsCFF="true", unicodeRange="U+0020-U+007A")]
    private var FteCffTestFont:Class;

    public function Test() {
        Font.registerFont(FteCffTestFont);

        trace("compatible normal: " + FontDescription.isFontCompatible("FteCffTest", FontWeight.NORMAL, FontPosture.NORMAL));
        trace("compatible missing: " + FontDescription.isFontCompatible("MissingFteCffTest", FontWeight.NORMAL, FontPosture.NORMAL));
        trace("compatible comma: " + FontDescription.isFontCompatible("MissingFteCffTest, FteCffTest", FontWeight.NORMAL, FontPosture.NORMAL));

        dumpLine("embedded", FontLookup.EMBEDDED_CFF);
    }

    private function dumpLine(label:String, lookup:String):void {
        var fd:FontDescription = new FontDescription("FteCffTest", FontWeight.NORMAL, FontPosture.NORMAL, lookup);
        var ef:ElementFormat = new ElementFormat(fd, 18);
        var tb:TextBlock = new TextBlock(new TextElement("iiiiiiiiii WWWWWWWWWW", ef));
        var line:TextLine = tb.createTextLine(null, 80);
        trace(label + " raw=" + line.rawTextLength + " width=" + line.textWidth.toFixed(2));
    }
}
}
