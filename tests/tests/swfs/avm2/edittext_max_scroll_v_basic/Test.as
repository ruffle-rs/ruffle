package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    public function Test() {
        stage.scaleMode = "noScale";

        for (var h = 0; h < 50; ++h) {
            for (var c = 0; c < 20; ++c) {
                test(h, c);
            }
        }
    }

    private function test(height: int, chars: uint):void {
        var text = new TextField();
        text.multiline = true;
        text.border = true;
        text.width = 100;
        text.height = height;
        text.embedFonts = true;
        var tf = new TextFormat();
        tf.font = "TestFont";
        tf.size = 20;
        tf.leading = 6;
        text.defaultTextFormat = tf;

        var i = chars;
        while (i > 1) {
            text.text += "a\n";
            i -= 2;
        }
        if (i > 0) {
            text.text += "a";
        }
        trace("h=" + height + ",chars=" + chars + ",maxScrollV=" + text.maxScrollV + ",textHeight=" + text.textHeight);
    }
}
}
