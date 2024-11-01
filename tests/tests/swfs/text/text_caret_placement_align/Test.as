package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;
import flash.events.MouseEvent;

[SWF(width="600", height="800")]
public class Test extends Sprite {
    [Embed(source="NotoSans.ttf", fontName="Noto Sans", embedAsCFF="false", unicodeRange="U+0020-U+007A")]
    private var font:Class;

    private var nextX:int = 1;
    private var nextY:int = 1;

    public function Test() {
        stage.scaleMode = "noScale";
        addChild(newTextField("left"));
        addChild(newTextField("right"));
        addChild(newTextField("center"));
        addChild(newTextField("justify"));
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
            trace("Caret placed at: " + begin + " in " + focus.name);
        }
    }

    private function newTextField(align:String): TextField {
        var text:TextField = new TextField();
        text.name = "text_" + align;
        text.border = true;
        text.type = "input";
        text.width = 600 - 3;
        text.height = 200 - 3;
        text.x = nextX;
        text.embedFonts = true;
        text.wordWrap = true;
        var tf:TextFormat = new TextFormat();
        tf.font = "Noto Sans";
        tf.size = 20;
        tf.align = align;
        tf.leading = 5;
        text.defaultTextFormat = tf;

        text.y = nextY;
        text.height = 100;
        text.multiline = true;
        text.text = "This is an example text\nand this is its second line";

        if (align == "justify") {
            text.text = "This is an example text which is long in order to wrap to the next line";
        }

        nextY += text.height + 2;
        return text;
    }

}
}
