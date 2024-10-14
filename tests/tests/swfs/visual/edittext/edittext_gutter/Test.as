package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

[SWF(width="100", height="110")]
public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0062")]
    private var testFont:Class;

    private var nextX:int = 2;
    private var nextY:int = 2;

    public function Test() {
        stage.scaleMode = "noScale";

        newTextField("a", 0, 0);
        newTextField("a", 1, 0);
        newTextField("a", 2, 0);
        newTextField("a", 3, 0);
        newTextField("a", 4, 0);
        newTextField("a", 5, 0);
        newTextField("a", 6, 0);
        newTextField("a", 7, 0);
        newTextField("a", 8, 0);
        newTextField("a", 40, 0);
        nextY += 34;
        nextX = 2;

        newTextField("b", 0, 0);
        newTextField("b", 1, 0);
        newTextField("b", 2, 0);
        newTextField("b", 3, 0);
        newTextField("b", 4, 0);
        newTextField("b", 5, 0);
        newTextField("b", 6, 0);
        newTextField("b", 7, 0);
        newTextField("b", 8, 0);
        newTextField("b", 40, 0);
        nextY += 34;
        nextX = 2;

        newTextField("b", 6, 0);
        newTextField("b", 6, 1);
        newTextField("b", 6, 2);
        newTextField("b", 6, 3);
        newTextField("b", 6, 4);
        newTextField("b", 6, 5);
        newTextField("b", 6, 6);
        newTextField("b", 6, 18);
        newTextField("b", 6, 19);
        newTextField("b", 6, 20);
        newTextField("b", 6, 21);
        newTextField("b", 6, 22);
    }

    private function newTextField(value: String, width: int, hscroll: int): void {
        var text:TextField = new TextField();
        text.border = true;
        text.width = width;
        text.x = nextX;
        text.y = nextY;
        text.height = 30;
        text.embedFonts = true;
        var tf = new TextFormat();
        tf.font = "TestFont";
        tf.size = 20;
        text.defaultTextFormat = tf;
        text.text = value;
        text.scrollH = hscroll;
        nextX += text.width + 2;
        addChild(text);
    }

}
}
