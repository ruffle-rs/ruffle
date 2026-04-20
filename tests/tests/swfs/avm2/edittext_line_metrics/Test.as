package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;
import flash.text.TextLineMetrics;

[SWF(width="300", height="300")]
public class Test extends Sprite {
    [Embed(source="NotoSans.ttf", fontName="Noto Sans", embedAsCFF="false", unicodeRange="U+0020-U+007E")]
    private var notoSans:Class;

    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var testFont:Class;

    [Embed(source="TestFontDefault.ttf", fontName="TestFontDefault", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var testFontDefault:Class;

    private var text:TextField;

    public function Test() {
        stage.scaleMode = "noScale";
        text = new TextField();
        text.border = true;
        text.x = 10;
        text.y = 10;
        text.width = 280;
        text.height = 280;
        text.multiline = true;
        text.embedFonts = true;
        text.wordWrap = true;
        var tf = new TextFormat();
        tf.font = "TestFontDefault";
        text.defaultTextFormat = tf;
        addChild(text);

        text.htmlText = "";
        text.htmlText += "<font face='TestFont'>acbd</font>\n";
        text.htmlText += "<font face='Noto Sans'>acbd</font>\n";
        text.htmlText += "\n";
        text.htmlText += "<font face='TestFont'>acbd</font><font face='Noto Sans'>mixed fonts</font>\n";
        text.htmlText += "<font face='Noto Sans'><font size='+2'>mixed</font> <font size='-2'>sizes</font></font>\n";
        text.htmlText += "<textformat leading='5'><font face='TestFont'>acbd</font><font face='Noto Sans'>leading</font></textformat>\n";
        text.htmlText += "<textformat leading='4'><font face='Noto Sans'>line so long that it will need to break at some point</font> <font face='TestFont'>acbd</font></textformat>\n";
        // TODO add cases for alignment (when it's fixed)

        for (var i = -1; i <= 9; ++i) {
            trace("text.getLineMetrics(" + i + ") = " + testMetrics(i));
        }
    }

    private function testMetrics(i:int):String {
        try {
            return metricsToStr(text.getLineMetrics(i));
        } catch(e) {
            return "" + e;
        }
        return "";
    }

    private function metricsToStr(metrics:TextLineMetrics):String {
        return "ascent=" + metrics.ascent + ", descent=" + metrics.descent + ", leading=" + metrics.leading + ", width=" + metrics.width + ", height=" + metrics.height + ", x=" + metrics.x;
    }
}
}
