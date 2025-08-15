package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;
import flash.events.MouseEvent;

[SWF(width="600", height="200")]
public class Test extends Sprite {
    [Embed(source="NotoSans.ttf", fontName="Noto Sans", embedAsCFF="false", unicodeRange="U+0020-U+007A")]
    private var font:Class;

    public function Test() {
        stage.scaleMode = "noScale";
        addChild(newTextField());
        stage.addEventListener(MouseEvent.MOUSE_UP, mouseUp);
    }

    private function mouseUp(event:MouseEvent):void {
        var focus = TextField(stage.focus);
        var begin = focus.selectionBeginIndex;
        var end = focus.selectionEndIndex;
        if (begin == end) {
            trace("    { \"type\": \"MouseMove\", \"pos\": [" + event.stageX + ", " + event.stageY + "] },");
            trace("    { \"type\": \"MouseDown\", \"pos\": [" + event.stageX + ", " + event.stageY + "], \"btn\": \"Left\" },");
            trace("    { \"type\": \"MouseUp\", \"pos\": [" + event.stageX + ", " + event.stageY + "], \"btn\": \"Left\" },");
            trace("Caret placed at: " + begin);
        }
    }

    private function newTextField(): TextField {
        var text:TextField = new TextField();
        text.border = true;
        text.type = "input";
        text.width = 600 - 3;
        text.height = 200 - 3;
        text.x = 1;
        text.embedFonts = true;
        var tf:TextFormat = new TextFormat();
        tf.font = "Noto Sans";
        tf.size = 20;
        tf.leading = 5;
        text.defaultTextFormat = tf;

        text.y = 1;
        text.height = 100;
        text.multiline = true;
        text.text = "This is an example text\nand this is its second line, which should be really long so that it's longer than the width\nand this is its third line\nand fourth!";

        text.scrollV = 2;
        text.scrollH = 50;

        return text;
    }

}
}
