package {
import flash.display.*;
import flash.text.*;
import flash.geom.*;

[SWF(width="200", height="200")]
public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    public function Test() {
        stage.scaleMode = "noScale";

        testWrap(false);
        testWrap(false, 2);
        testWrap(false, 1, 2);
        testWrap(false, 2, 2);
        testWrap(true);
        testWrap(true, 2);
        testWrap(true, 1, 2);
        testWrap(true, 2, 2);

        textAlign("center", false);
        textAlign("center", false, 2);
        textAlign("center", false, 1, 2);
        textAlign("center", false, 2, 2);
        textAlign("center", true);
        textAlign("center", true, 2);
        textAlign("center", true, 1, 2);
        textAlign("center", true, 2, 2);

        textAlign("right", false);
        textAlign("right", false, 2);
        textAlign("right", false, 1, 2);
        textAlign("right", false, 2, 2);
        textAlign("right", true);
        textAlign("right", true, 2);
        textAlign("right", true, 1, 2);
        textAlign("right", true, 2, 2);

        autoSize("left", false);
        autoSize("left", false, 2);
        autoSize("left", false, 1, 2);
        autoSize("left", false, 2, 2);
        autoSize("left", true);
        autoSize("left", true, 2);
        autoSize("left", true, 1, 2);
        autoSize("left", true, 2, 2);

        autoSize("center", false);
        autoSize("center", false, 2);
        autoSize("center", false, 1, 2);
        autoSize("center", false, 2, 2);
        autoSize("center", true);
        autoSize("center", true, 2);
        autoSize("center", true, 1, 2);
        autoSize("center", true, 2, 2);

        autoSize("right", false);
        autoSize("right", false, 2);
        autoSize("right", false, 1, 2);
        autoSize("right", false, 2, 2);
        autoSize("right", true);
        autoSize("right", true, 2);
        autoSize("right", true, 1, 2);
        autoSize("right", true, 2, 2);
    }

    private function testWrap(device: Boolean, scaleX: Number = 1, scaleY: Number = 1):void {
        trace("Word wrap, device=" + device + ", scaleX=" + scaleX + ", scaleY=" + scaleY);
        var text:TextField = new TextField();
        text.width = 60;
        text.height = 60;
        text.embedFonts = !device;
        var tf:TextFormat = new TextFormat();
        tf.font = "TestFont";
        tf.size = 10;
        text.defaultTextFormat = tf;

        text.multiline = true;
        text.text = "abababab";
        text.scaleX = scaleX;
        text.scaleY = scaleY;
        text.wordWrap = true;

        for (var i:int = 0; i < 8; ++i) {
            trace("  " + i + ": " + text.getCharBoundaries(i));
        }
    }

    private function textAlign(align:String, device: Boolean, scaleX: Number = 1, scaleY: Number = 1):void {
        trace("Text align, align=" + align + ", device=" + device + ", scaleX=" + scaleX + ", scaleY=" + scaleY);
        var text:TextField = new TextField();
        text.width = 60;
        text.height = 60;
        text.embedFonts = !device;
        var tf:TextFormat = new TextFormat();
        tf.font = "TestFont";
        tf.size = 10;
        tf.align = align;
        text.defaultTextFormat = tf;

        text.text = "ab";
        text.scaleX = scaleX;
        text.scaleY = scaleY;

        for (var i:int = 0; i < 2; ++i) {
            trace("  " + i + ": " + text.getCharBoundaries(i));
        }
    }

    private function autoSize(autoSize:String, device: Boolean, scaleX: Number = 1, scaleY: Number = 1):void {
        trace("Auto size, autoSize=" + autoSize + ", device=" + device + ", scaleX=" + scaleX + ", scaleY=" + scaleY);
        var text:TextField = new TextField();
        text.width = 60;
        text.height = 60;
        text.embedFonts = !device;
        var tf:TextFormat = new TextFormat();
        tf.font = "TestFont";
        tf.size = 10;
        text.defaultTextFormat = tf;

        text.text = "ab";
        text.scaleX = scaleX;
        text.scaleY = scaleY;
        text.autoSize = autoSize;

        trace("  x: " + text.x);
        trace("  y: " + text.y);
        trace("  width: " + text.width);
        trace("  height: " + text.height);
        for (var i:int = 0; i < 2; ++i) {
            trace("  " + i + ": " + text.getCharBoundaries(i));
        }
    }
}
}
