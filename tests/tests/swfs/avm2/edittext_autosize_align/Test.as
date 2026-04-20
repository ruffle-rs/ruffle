package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

[SWF(width="200", height="800")]
public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    private var nextY: int = 0;
    private var nextX: int = 0;

    public function Test() {
        stage.scaleMode = "noScale";

        newTextField("none", "left", false);
        newTextField("none", "center", false);
        newTextField("none", "right", false);
        newTextField("left", "left", false);
        newTextField("left", "center", false);
        newTextField("left", "right", false);
        newTextField("center", "left", false);
        newTextField("center", "center", false);
        newTextField("center", "right", false);
        newTextField("right", "left", false);
        newTextField("right", "center", false);
        newTextField("right", "right", false);

        nextY = 0;
        nextX = 100;
        newTextField("none", "left", true);
        newTextField("none", "center", true);
        newTextField("none", "right", true);
        newTextField("left", "left", true);
        newTextField("left", "center", true);
        newTextField("left", "right", true);
        newTextField("center", "left", true);
        newTextField("center", "center", true);
        newTextField("center", "right", true);
        newTextField("right", "left", true);
        newTextField("right", "center", true);
        newTextField("right", "right", true);
    }

    private function newTextField(autosize: String, align: String, wordWrap: Boolean):void {
        var text = new TextField();
        text.multiline = true;
        text.wordWrap = wordWrap;
        text.border = true;
        text.x = nextX;
        text.y = nextY;
        nextY += 62;
        text.width = 100;
        text.height = 60;
        text.embedFonts = true;
        var tf = new TextFormat();
        tf.font = "TestFont";
        tf.size = 20;
        tf.align = align;
        tf.leading = 5;
        text.defaultTextFormat = tf;
        text.text = "ab\nabab";
        text.autoSize = autosize;
        addChild(text);
    }
}
}
