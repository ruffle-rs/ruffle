package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;
import flash.text.TextLineMetrics;

[SWF(width="300", height="300")]
public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0020,U+0061-U+0064")]
    private var testFont:Class;

    private var nextX = 1;
    private var nextY = 1;

    public function Test() {
        stage.scaleMode = "noScale";

        newText(280);
        newText(260);
        newText(230);
        newText(200);
        newText(160);
        newText(140);
    }

    private function newText(width: int) {
        var text = new TextField();
        text.border = true;
        text.x = nextX;
        text.y = nextY;
        text.width = width;
        text.height = 36;
        nextY += text.height + 2;
        text.multiline = true;
        text.embedFonts = true;
        text.wordWrap = true;
        var tf = new TextFormat();
        tf.font = "TestFont";
        tf.size = 10;
        tf.align = "justify";
        text.defaultTextFormat = tf;
        text.text = "ac acac acacac acacacac acacacacac";
        addChild(text);
    }
}
}
