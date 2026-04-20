package {
import flash.display.*;
import flash.text.*;

[SWF(width="100", height="50")]
public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    public function Test() {
        stage.scaleMode = "noScale";

        addTextField(0, 0, false);
        addTextField(0, 25, true);
    }

    private function addTextField(x:Number, y:Number, kerning:Boolean):void {
        var field:TextField = new TextField();
        field.type = "input";
        field.width = 100;
        field.x = x;
        field.height = 25;
        field.y = y;
        field.border = true;

        var tf:TextFormat = new TextFormat("TestFont", 10);
        tf.kerning = kerning;
        field.defaultTextFormat = tf;

        field.text = "abcd";

        var lm = field.getLineMetrics(0);
        trace("LM width: " + lm.width);
        trace("width: " + field.textWidth);

        addChild(field);
    }
}
}
