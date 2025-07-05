package {
import flash.display.*;
import flash.text.*;

[SWF(width="100", height="200")]
public class Test extends Sprite {
    public function Test() {
        stage.scaleMode = "noScale";
        testFont();
    }

    private function testFont() {
        var tf;

        tf = new TextField();
        tf.defaultTextFormat = new TextFormat("TestFontA", 10);
        tf.text = "a";
        testTextField(tf, -1);

        tf = new TextField();
        tf.defaultTextFormat = new TextFormat("TestFontA", 10);
        tf.text = "ab";
        testTextField(tf, 1);

        tf = new TextField();
        tf.defaultTextFormat = new TextFormat("TestFontA", 10);
        tf.text = "b";
        testTextField(tf, 0);
    }

    private function testTextField(tf: TextField, fallbackGlyph: int) {
        trace("Text: " + tf.text);

        trace("  Text runs:");
        printTextRuns(tf.getTextRuns());

        trace("  Character bounds:");
        var i = 0;
        while (i < 2) {
            var bounds = tf.getCharBoundaries(i);
            if (bounds == null) break;
            if (i != fallbackGlyph) {
                trace("    " + i + ": x=" + bounds.x + ", y=" + bounds.y + ", width=" + bounds.width + ", height=" + bounds.height);
            } else {
                trace("    " + i + ":");
                trace("      x=" + bounds.x + ", y=" + bounds.y);
                trace("      width is zero? " + (bounds.width == 0));
                trace("      height is zero? " + (bounds.height == 0));
            }
            ++i;
        }

        trace("  Line metrics:");
        var metrics = tf.getLineMetrics(0);
        trace("    ascent=" + metrics.ascent + ", descent=" + metrics.descent + ", height=" + metrics.height + ", leading=" + metrics.leading);
        if (fallbackGlyph < 0) {
            trace("    width=" + metrics.width);
        } else {
            trace("    width is zero? " + (metrics.width == 0));
            trace("    width is 10? " + (metrics.width == 10));
        }
    }

    private function printTextRuns(runs: Array) {
        trace("    Text runs (" + runs.length + "):");
        for each (var run in runs) {
            trace("    from " + run.beginIndex + " to " + run.endIndex + ": " + describeTextFormat(run.textFormat));
        }
    }

    private function describeTextFormat(tf: TextFormat): String {
        return "size=" + tf.size +
                ", blockIndent=" + tf.blockIndent +
                ", font=" + tf.font +
                ", align=" + tf.align +
                ", leading=" + tf.leading +
                ", display=" + tf.display +
                ", kerning=" + tf.kerning +
                ", leftMargin=" + tf.leftMargin +
                ", rightMargin=" + tf.rightMargin +
                ", color=" + tf.color +
                ", bold=" + tf.bold +
                ", italic=" + tf.italic +
                ", bullet=" + tf.bullet +
                ", underline=" + tf.underline;
    }

    private function escape(string: String): String {
        return string.replace(/\n/g, "\\n").replace(/\r/g, "\\r").replace(/Times New Roman/g, "Times");
    }
}
}
