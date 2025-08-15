package {
import flash.display.*;
import flash.text.*;
import flash.geom.*;
import flash.utils.*;

/**
 * In this test we're doing:
 *  1. draw (GPU)
 *  2. full fill (CPU)
 *  3. draw (GPU)
 *
 * And we're verifying whether these operations are properly
 * synchronized, especially the "CPU overwriting GPU" part (1 vs 2).
 */
[SWF(width="50", height="50")]
public class Test extends MovieClip {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    public function Test() {
        var bd = new BitmapData(50, 50);

        var tf = new TextField();
        tf.defaultTextFormat = new TextFormat("TestFont", 20);
        tf.embedFonts = true;
        tf.text = "ab";

        var mx = new Matrix();
        bd.draw(tf, mx);
        bd.fillRect(new Rectangle(0, 0, 100, 100), 0);

        mx.translate(20, 0);
        bd.draw(tf, mx);

        addChild(new Bitmap(bd));
    }
}
}
