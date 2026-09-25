package {
import flash.display.*;
import flash.text.*;
import flash.geom.*;
import flash.events.*;

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
        tf.text = "";
        for (var i = 0; i < 20; ++i) {
            tf.appendText("abababababababababababababababababababab\n");
        }
        tf.setSelection(0, 0);

        tf.addEventListener("scroll", function(evt:Event):void {
            trace("scroll: " + evt);
            trace("  tf.scrollH=" + tf.scrollH);
            trace("  tf.scrollV=" + tf.scrollV);
            evt.preventDefault();
        });

        trace("Set to 2");
        tf.scrollV = 2;
        trace("Set to 2");
        tf.scrollV = 2;

        trace("Set to 0");
        tf.scrollV = 0;
        trace("Set to -1");
        tf.scrollV = -1;

        trace("Set to 2");
        tf.scrollV = 2;
        trace("Set to -1");
        tf.scrollV = -1;

        trace("Set to 1");
        tf.scrollV = 1;

        addChild(tf);
    }
}
}
