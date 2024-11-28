package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    public function Test() {
        stage.scaleMode = "noScale";

        // h scroll when the longest line is below the window
        test(50, -1, false);
        test(50, -1, true);

        for (var w = 0; w < 30; ++w) {
            for (var c = 0; c < 15; ++c) {
                test(w, c, false);
                test(w, c, true);
            }
        }

        // some higher, more extreme values
        var widths = [50, 200, 600, 1200];
        var chars = [0, 1, 10, 50, 400, 1200];
        for each (var w in widths) {
            for each (var c in chars) {
                test(w, c, false);
                test(w, c, true);
            }
        }
    }

    private function test(width: int, chars: int, input: Boolean):void {
        if(input) return;
        var text = new TextField();
        var text2 = new TextField();
        text.type = "input";
        text.multiline = true;
        text2.multiline = true;
        text.border = true;
        text2.border = true;
        text.width = width;
        text2.width = width;
        text.height = 50;
        text2.height = 50;
        text.embedFonts = true;
        text2.embedFonts = true;
        var tf = new TextFormat();
        tf.font = "TestFont";
        tf.size = 20;
        tf.leading = 6;
        text.defaultTextFormat = tf;
        text2.defaultTextFormat = tf;

        var i = chars;
        while (i > 0) {
            text.text += "a";
            text2.text += "a";
            i -= 1;
        }

        if (chars < 0) {
            text.text = "aaaaa\n\n\n\n\n\naaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
            text2.text = "aaaaa\n\n\n\n\n\naaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        }

        trace("w=" + width + ",chars=" + chars + ",input=" + input + ",maxScrollHDiff=" + (text.maxScrollH - text2.maxScrollH) + ",maxScrollH=" + text.maxScrollH + ",maxScrollH2=" + text2.maxScrollH + ",textWidth=" + text.textWidth);
    }
}
}
