package {
import flash.display.Sprite;
import flash.geom.Rectangle;
import flash.text.TextField;
import flash.text.TextFieldAutoSize;
import flash.text.TextFormat;

// SmallTextFont and PlainFont are the same font, except that SmallTextFont
// has the SmallText flag set. They are added to test.swf by add_fonts.py.
public class Test extends Sprite {
    private var nextY:Number = 0;

    public function Test() {
        trace("// Glyph advances");
        for each (var font:String in ["SmallTextFont", "PlainFont"]) {
            for each (var size:Number in [7, 10, 15, 20]) {
                for each (var text:String in ["aaaaaaaaaa", "abcd abcd"]) {
                    report(font + " " + size + " '" + text + "'", field(font, size, text));
                }
            }
        }

        trace("// letterSpacing");
        for each (var spacing:Number in [0.3, 0.5, 1.5, -0.5, -1]) {
            report("SmallTextFont 20 letterSpacing " + spacing, field("SmallTextFont", 20, "aaaaaaaaaa", spacing));
            report("PlainFont 20 letterSpacing " + spacing, field("PlainFont", 20, "aaaaaaaaaa", spacing));
        }

        trace("// kerning");
        report("SmallTextFont 20 kerning", field("SmallTextFont", 20, "kkkkkkkkkk", null, true));
        report("SmallTextFont 20 kerning letterSpacing 0.45", field("SmallTextFont", 20, "kkkkkkkkkk", 0.45, true));
        report("PlainFont 20 kerning", field("PlainFont", 20, "kkkkkkkkkk", null, true));

        trace("// Two formats in one line");
        var sizes:TextField = field("SmallTextFont", 10, "aaaaaaaaaa");
        sizes.setTextFormat(new TextFormat(null, 20), 5, 10);
        report("SmallTextFont 10 then 20", sizes);
        var fonts:TextField = field("SmallTextFont", 20, "aaaaaaaaaa");
        fonts.setTextFormat(new TextFormat("PlainFont"), 5, 10);
        report("SmallTextFont then PlainFont 20", fonts);

        trace("// Scaled field");
        var scaled:TextField = field("SmallTextFont", 10, "aaaaaaaaaa");
        scaled.scaleX = 2;
        scaled.scaleY = 2;
        report("SmallTextFont 10 scale 2", scaled);

        trace("// autoSize");
        var auto:TextField = field("SmallTextFont", 10, "aaaaaaaaaa");
        auto.autoSize = TextFieldAutoSize.LEFT;
        trace("SmallTextFont 10 autoSize: width " + auto.width);

        trace("// getCharBoundaries");
        var chars:TextField = field("SmallTextFont", 20, "abcd");
        for (var i:int = 0; i < chars.length; i++) {
            var bounds:Rectangle = chars.getCharBoundaries(i);
            trace("SmallTextFont 20 'abcd' char " + i + ": x " + bounds.x + ", width " + bounds.width);
        }

        // Advanced anti-aliasing has its own glyph metrics, which Ruffle does not
        // implement, so only compare SmallTextFont with PlainFont here.
        trace("// antiAliasType advanced");
        for each (size in [10, 20]) {
            var advancedSmall:TextField = field("SmallTextFont", size, "aaaaaaaaaa", null, null, "advanced");
            var advancedPlain:TextField = field("PlainFont", size, "aaaaaaaaaa", null, null, "advanced");
            trace("SmallTextFont " + size + " advanced, same textWidth as PlainFont: " + (advancedSmall.textWidth == advancedPlain.textWidth));
        }
        var changed:TextField = field("SmallTextFont", 10, "aaaaaaaaaa");
        changed.antiAliasType = "advanced";
        trace("SmallTextFont 10 set to advanced after layout: textWidth " + changed.textWidth);
    }

    private function field(font:String, size:Number, text:String, letterSpacing:* = null, kerning:* = null, antiAliasType:String = null):TextField {
        var tf:TextField = new TextField();
        tf.embedFonts = true;
        if (antiAliasType != null) {
            tf.antiAliasType = antiAliasType;
        }
        tf.width = 400;
        tf.height = size + 8;
        tf.y = nextY;
        nextY += size + 2;
        var format:TextFormat = new TextFormat(font, size);
        if (letterSpacing != null) {
            format.letterSpacing = letterSpacing;
        }
        if (kerning != null) {
            format.kerning = kerning;
        }
        tf.defaultTextFormat = format;
        tf.text = text;
        addChild(tf);
        return tf;
    }

    private function report(label:String, tf:TextField):void {
        trace(label + ": textWidth " + tf.textWidth + ", line width " + tf.getLineMetrics(0).width);
    }
}
}
