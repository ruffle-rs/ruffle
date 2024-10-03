package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

[SWF(width="400", height="400")]
public class Test extends Sprite {
    [Embed(source="NotoSans.ttf", fontName="Noto Sans", embedAsCFF="false", unicodeRange="U+0020-U+007E")]
    private var notoSans:Class;

    private var text:TextField;

    public function Test() {
        stage.scaleMode = "noScale";
        text = new TextField();
        text.border = true;
        text.x = 10;
        text.y = 10;
        text.width = 380;
        text.height = 380;
        text.multiline = true;
        text.embedFonts = true;
        var tf = new TextFormat();
        tf.font = "Noto Sans";
        tf.size = 12;
        tf.leading = 4;
        text.defaultTextFormat = tf;
        addChild(text);

        // TODO Add cases with multiple lines when leading is fixed.
        testHtml("test\n");
        testHtml("ay .,\n");
        testHtml("x<font size='-2'>x<font size='+1'>x</font></font>");
        testHtml("<li>i</li>", 2);
        testHtml("<p align='right'>test</p>", 2);
        testHtml("l1\nl2\n", 0, 2);
    }

    private function testHtml(htmlText:String, xPrecision:int = 0, yPrecision:int = 0):void {
        text.htmlText = htmlText;
        trace("Text: " + htmlText.replace(/[\r\n]/g, "\\n"));

        testAt(-5, xPrecision, yPrecision);
        testAt(-1, xPrecision, yPrecision);
        for (var i = 0; i <= text.text.length; ++i) {
            testAt(i, xPrecision, yPrecision);
        }
    }

    private function testAt(charIndex:int, xPrecision:int, yPrecision:int):void {
        var bounds = text.getCharBoundaries(charIndex);
        if (bounds != null) {
            // TODO These precision-related calculations should be deleted.
            //   They are present here only because currently Ruffle
            //   is slightly off in its font rendering.
            if (xPrecision > 0) {
                bounds.x = Math.ceil(bounds.x / xPrecision) * xPrecision;
            }
            if (yPrecision > 0) {
                bounds.y = Math.floor(bounds.y / yPrecision) * yPrecision;
            }
        }
        trace("  text.getCharBoundaries(" + charIndex + ") = " + bounds);
    }
}
}
