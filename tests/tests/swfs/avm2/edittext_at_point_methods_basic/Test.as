package {
import flash.display.Sprite;
import flash.events.MouseEvent;
import flash.text.TextField;
import flash.text.TextFormat;

public class Test extends Sprite {
    [Embed(source="NotoSans.ttf", fontName="Noto Sans", embedAsCFF="false", unicodeRange="U+0020-U+007E")]
    private var notoSans: Class;

    private var text1: TextField;

    public function Test() {
        var tf = new TextFormat();
        tf.font = "Noto Sans";
        tf.size = 30;

        text1 = new TextField();
        text1.defaultTextFormat = tf;
        text1.border = true;
        text1.x = 20;
        text1.y = 20;
        text1.width = 200;
        text1.text = "MXNP";

        addChild(text1);

        text1.addEventListener(MouseEvent.CLICK, function (evt: MouseEvent): void {
            trace("Mouse down at: " + evt.stageX + ", " + evt.stageY);
            printValues("text1", text1, evt.localX, evt.localY);
        });
    }

    private function printValues(name:String, text:TextField, x: Number, y: Number):void {
        trace("Clicked: " + name);
        trace("  " + name + ".getCharIndexAtPoint(" + x + ", " + y + ") = " + text.getCharIndexAtPoint(x, y));
        trace("  " + name + ".getLineIndexAtPoint(" + x + ", " + y + ") = " + text.getLineIndexAtPoint(x, y));
    }
}
}
