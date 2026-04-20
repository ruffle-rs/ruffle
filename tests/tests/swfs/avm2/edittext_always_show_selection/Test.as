package {
import flash.display.*;
import flash.text.*;

[SWF(width="50", height="110")]
public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    private var text1: TextField;
    private var text2: TextField;
    private var text3: TextField;
    private var text4: TextField;

    public function Test() {
        text1 = newTextField();
        text1.x = 0;
        text1.y = 0;
        text1.width = 50;
        text1.height = 20;
        text1.text = "ababa";
        text1.setSelection(1, 4);

        text2 = newTextField();
        text2.x = 0;
        text2.y = 30;
        text2.width = 50;
        text2.height = 20;
        text2.text = "ababa";
        text2.alwaysShowSelection = true;
        text2.setSelection(1, 4);

        text3 = newTextField();
        text3.x = 0;
        text3.y = 60;
        text3.width = 50;
        text3.height = 20;
        text3.type = "input";
        text3.text = "ababa";
        text3.alwaysShowSelection = true;
        text3.setSelection(2, 2);

        text4 = newTextField();
        text4.x = 0;
        text4.y = 90;
        text4.width = 50;
        text4.height = 20;
        text4.text = "ababa";
        text4.alwaysShowSelection = true;
        text4.setSelection(1, 4);

        stage.focus = text4;
    }

    private function newTextField(): TextField {
        var text:TextField = new TextField();
        text.embedFonts = true;
        text.defaultTextFormat = new TextFormat("TestFont", 10);
        addChild(text);
        return text;
    }
}
}
