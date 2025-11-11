package {
import flash.display.*;
import flash.text.*;
import flash.geom.*;

[SWF(width="200", height="200")]
public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    private var nextX:int = 0;
    private var nextY:int = 0;

    public function Test() {
        stage.scaleMode = "noScale";

        test(false);
        test(false, 2);
        test(false, 1, 2);
        test(false, 2, 2);

        nextX = 100;
        nextY = 0;

        test(true);
        test(true, 2);
        test(true, 1, 2);
        test(true, 2, 2);
    }

    private function test(device: Boolean, scaleX: Number = 1, scaleY: Number = 1):TextField {
        var text:TextField = new TextField();
        text.x = nextX;
        text.y = nextY;
        text.border = true;
        text.width = 40;
        text.height = 20;
        text.embedFonts = !device;
        var tf:TextFormat = new TextFormat();
        tf.font = "TestFont";
        tf.size = 10;
        text.defaultTextFormat = tf;

        text.multiline = true;
        text.text = "ab\n\n\n\n\n\n\n\n\n\n\nab";
        text.scaleX = scaleX;
        text.scaleY = scaleY;
        addChild(text);

        nextY += 50;

        trace("" + device + " = " + text.getCharBoundaries(0) + "," + text.getCharBoundaries(1));
        trace("" + device + " = " +
            text.getCharIndexAtPoint(1, 5) + "," +
            text.getCharIndexAtPoint(2, 5) + "," +
            text.getCharIndexAtPoint(5, 5) + "," +
            text.getCharIndexAtPoint(8, 5) + "," +
            text.getCharIndexAtPoint(10, 5) + "," +
            text.getCharIndexAtPoint(12, 5) + "," +
            text.getCharIndexAtPoint(15, 5) + "," +
            text.getCharIndexAtPoint(8, 1) + "," +
            text.getCharIndexAtPoint(12, 1) + "," +
            text.getCharIndexAtPoint(8, 12) + "," +
            text.getCharIndexAtPoint(12, 12) + "," +
            "");
        trace("" + device + " = " + metricsToString(text.getLineMetrics(0)));
        trace("" + device + " = " + text.textHeight);
        trace("" + device + " = " + text.textWidth);

        return text;
    }

    private function metricsToString(m:TextLineMetrics): String {
        return "height=" + Math.round(m.height) +
            ",width=" + Math.round(m.width) +
            ",x=" + Math.round(m.x) +
            ",ascent=" + Math.round(m.ascent) +
            ",descent=" + Math.round(m.descent);
    }
}
}
