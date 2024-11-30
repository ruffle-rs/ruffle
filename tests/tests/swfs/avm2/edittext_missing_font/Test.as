package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

[SWF(width="200", height="200")]
public class Test extends Sprite {
    private var nextY: int = 0;
    private var nextX: int = 0;

    public function Test() {
        stage.scaleMode = "noScale";

        newTextField(true, "input");
        newTextField(true, "dynamic");
        newTextField(false, "input");
        newTextField(false, "dynamic");
    }

    private function newTextField(device: Boolean, type: String):void {
        var text = new TextField();
        text.type = type;
        text.border = true;
        text.x = nextX;
        text.y = nextY;
        nextY += 42;
        text.width = 100;
        text.height = 40;
        text.embedFonts = !device;
        var tf = new TextFormat();
        tf.font = "Unknown Font 6ad5511bcd8b089c25e2212243c819d1";
        tf.size = 20;
        tf.leading = 5;
        text.defaultTextFormat = tf;
        text.text = "xyz";
        addChild(text);

        trace("device=" + device + ", type=" + type);
        trace("  getTextFormat(0, 1).font=" + (text.getTextFormat(0, 1).font));
        trace("  length=" + text.length);
        trace("  text=" + text.text);
        trace("  numLines=" + text.numLines);
        trace("  textWidth is zero?=" + (text.textWidth == 0));
        trace("  getCharBoundaries(0) is null?=" + (text.getCharBoundaries(0) == null));
        trace("  getCharBoundaries(1) is null?=" + (text.getCharBoundaries(1) == null));
        trace("  getCharBoundaries(3) is null?=" + (text.getCharBoundaries(3) == null));
        trace("  getLineMetrics(0).ascent is zero?=" + (text.getLineMetrics(0).ascent == 0));
        trace("  getLineMetrics(0).descent is zero?=" + (text.getLineMetrics(0).descent == 0));
        trace("  getLineMetrics(0).height is 5?=" + (text.getLineMetrics(0).height == 5));
        trace("  getLineMetrics(0).leading=" + (text.getLineMetrics(0).leading));
        trace("  getLineMetrics(0).width is zero?=" + (text.getLineMetrics(0).width == 0));
    }
}
}
