package {
import flash.display.*;
import flash.text.*;

[SWF(width="100", height="50")]
public class Test extends Sprite {
    [Embed(source="TestFontNoInk.ttf", fontName="TestFontNoInk", embedAsCFF="false", unicodeRange="U+0020,U+0030-U+0031,U+0061-U+0064")]
    private var TestFontNoInk:Class;

    public function Test() {
        stage.scaleMode = "noScale";

        addTextField(0, 0, 10);
        addTextField(0, 25, 20);
    }

    private function addTextField(x:Number, y:Number, size:Number):void {
        var field:TextField = new TextField();
        field.width = 100;
        field.x = x;
        field.height = 25;
        field.y = y;
        field.border = true;

        field.defaultTextFormat = new TextFormat("TestFontNoInk", size);
        field.text = "a b0c1d";

        var lm = field.getLineMetrics(0);
        trace("LM width: " + lm.width);
        trace("width: " + field.textWidth);
        for (var i = 0; i < 7; ++i) {
            trace("bounds " + i + ": " + field.getCharBoundaries(i));
        }

        addChild(field);
    }
}
}
