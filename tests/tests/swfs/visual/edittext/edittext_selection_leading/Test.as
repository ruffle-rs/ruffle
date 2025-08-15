package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;
import flash.events.KeyboardEvent;

[SWF(width="100", height="200")]
public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var testFont:Class;

    private var nextX:int = 2;
    private var nextY:int = 2;

    private var ignoreEsc:Boolean = true;
    private var selection:int = 0;

    public function Test() {
        stage.scaleMode = "noScale";

        // No leading
        newTextField(0);

        // Positive leading
        newTextField(10);

        // Negative leading
        newTextField(-10);

        nextSelect();
        stage.addEventListener("keyDown", function(evt:*):void {
            nextSelect();
        });
    }

    private function newTextField(leading: int): void {
        var text:TextField = new TextField();
        text.border = true;
        text.width = 96;
        text.x = nextX;
        text.y = nextY;
        text.multiline = true;
        text.height = 60;
        text.embedFonts = true;
        text.selectable = true;
        text.textColor = 0xFF00FF;
        text.background = true;
        text.backgroundColor = 0x00FFFF;
        var tf = new TextFormat();
        tf.font = "TestFont";
        tf.size = 20;
        tf.leading = leading;
        text.defaultTextFormat = tf;
        text.htmlText = "aacc\nacac";
        nextY += text.height + 2;
        addChild(text);
    }

    private function nextSelect(): void {
        trace("Selection: " + this.selection);
        var childId = this.selection / 4;
        var selType = this.selection % 4;
        this.selection += 1;

        var text = getChildAt(childId) as TextField;
        stage.focus = text;
        if (selType == 0) {
            text.setSelection(1, 3);
        } else if (selType == 1) {
            text.setSelection(6, 8);
        } else if (selType == 2) {
            text.setSelection(1, 7);
        } else if (selType == 3) {
            text.setSelection(0, 9);
        }
    }
}
}
