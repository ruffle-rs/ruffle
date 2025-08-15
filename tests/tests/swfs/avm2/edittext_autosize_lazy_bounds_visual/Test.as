package {
import flash.display.Sprite;
import flash.display.DisplayObject;
import flash.text.TextField;
import flash.text.TextFormat;
import flash.events.Event;

[SWF(width="200", height="200")]
public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    public function Test() {
        stage.scaleMode = "noScale";

        var text1 = new TextField();
        var text2 = new TextField();
        var text3 = new TextField();
        var text4 = new TextField();

        setUpText(text1);
        setUpText(text2);
        setUpText(text3);
        setUpText(text4);

        text1.x = 0;
        text1.y = 0;
        text2.x = 0;
        text2.y = 100;
        text3.x = 100;
        text3.y = 0;
        text4.x = 100;
        text4.y = 100;

        text1.autoSize = "right";
        text1.x;
        text1.autoSize = "left";

        text2.autoSize = "left";
        text2.x;
        text2.wordWrap = true;

        text3.autoSize = "right";
        text3.autoSize = "left";

        text4.autoSize = "left";
        text4.wordWrap = true;

        addChild(text1);
        addChild(text2);
        addChild(text3);
        addChild(text4);
    }

    private function setUpText(text: TextField):void {
        var tf = new TextFormat();
        tf.size = 20;
        tf.font = "TestFont";
        text.defaultTextFormat = tf;
        text.embedFonts = true;
        text.border = true;
        text.text = "aa";
        text.width = 100;
        text.height = 60;
    }
}
}
