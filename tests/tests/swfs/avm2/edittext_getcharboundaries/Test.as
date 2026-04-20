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

        testHtml("test");
        testHtml("ay .,\n");
        testHtml("x<font size='-2'>x<font size='+1'>x</font></font>");
        testHtml("<li>i</li>", 2);
        testHtml("<p align='right'>test</p>", 2);
        testHtml("l1\nl2\n", 0);
        testHtml("<li>i</li><li>j</li>", 2);
        testHtml("1\n2\n3\n4\n5", 0);
        testHtml("<p><font size='+2'>xyM</font></p><p><font size='+4'>xyM</font></p><p><font size='-2'>xyM</font></p>", 0);

        var tf = text.defaultTextFormat;
        tf.leading = 0;
        text.defaultTextFormat = tf;

        testHtml("<p><font size='+2'>xyM</font></p><p><font size='+4'>xyM</font></p><p><font size='-2'>xyM</font></p>", 0);

        text.wordWrap = true;
        testHtml("<p align='justify'>xxxx y zzzz xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx</p>", 0);
    }

    private function testHtml(htmlText:String, xPrecision:int = 0):void {
        text.htmlText = htmlText;
        trace("Text: " + htmlText.replace(/[\r\n]/g, "\\n"));

        testAt(-5, xPrecision);
        testAt(-1, xPrecision);
        for (var i = 0; i <= text.text.length; ++i) {
            testAt(i, xPrecision);
        }
    }

    private function testAt(charIndex:int, xPrecision:int):void {
        var bounds = text.getCharBoundaries(charIndex);
        if (bounds != null) {
            // TODO These precision-related calculations should be deleted.
            //   They are present here only because currently Ruffle
            //   is slightly off in its font rendering.
            if (xPrecision > 0) {
                bounds.x = Math.ceil(bounds.x / xPrecision) * xPrecision;
            }
        }
        trace("  text.getCharBoundaries(" + charIndex + ") = " + bounds);
    }
}
}
