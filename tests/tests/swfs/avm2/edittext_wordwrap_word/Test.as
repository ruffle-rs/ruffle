package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    public function Test() {
        runTests();
    }

    function runTests() {
        var text = new TextField();
        text.embedFonts = true;
        var tf = new TextFormat("TestFont", 10);
        tf.letterSpacing = 10;
        text.defaultTextFormat = tf;
        text.wordWrap = true;
        text.text = "aaaaa";

        text.height = 100;
        text.width = 100;

        for (var i = 0; i < 50; ++i) {
            trace("width = " + text.width);
            text.textWidth;
            trace("  numLines = " + text.numLines);
            trace("  line 1 = " + text.getLineText(0));

            text.width -= 1;
        }
    }
}
}
