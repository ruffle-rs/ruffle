package {
import flash.display.*;
import flash.text.*;
import flash.geom.*;

[SWF(width="100", height="100")]
public class Test extends Sprite {
    [Embed(source="SpaceAB.ttf", fontName="SpaceAB", embedAsCFF="false", unicodeRange="U+0020,U+0061-U+0062")]
    private var SpaceAB:Class;

    public function Test() {
        stage.scaleMode = "noScale";

        var tf:TextField = new TextField();
        tf.embedFonts = true;
        tf.defaultTextFormat = new TextFormat("SpaceAB", 20);
        tf.multiline = true;
        tf.x = 0;
        tf.y = 0;
        tf.width = 100;
        tf.height = 100;
        tf.border = true;
        tf.type = "input";
        tf.text = "  \n\n \n";

        addChild(tf);

        stage.focus = tf;
    }
}
}
