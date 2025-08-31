package {
import flash.display.*;
import flash.text.*;

public class Test extends Sprite {
    [Embed(source="NotoSans.ttf", fontName="Noto Sans", embedAsCFF="false", unicodeRange="U+0020-U+007E")]
    private var notoSans: Class;

    private var text1:TextField;
    private var text2:TextField;
    private var text3:TextField;

    public function Test() {
        text1 = newTextField();
        text1.text = "line 1\n line2\n\n line 4\n";
        text2 = newTextField();
        text2.htmlText = "<p>line 1 </p><br/><li>line 2 </li>\n\n<li> line 4</li>";
        text3 = newTextField();
        text3.wordWrap = true;
        text3.width = 50;
        text3.text = "first second\nthird";

        addChild(text1);
        addChild(text2);
        addChild(text3);
        printValues("text1", text1, 25);
        printValues("text2", text2, 25);
        printValues("text3", text3, 20);
    }

    private function printValues(name:String, text:TextField, len: int):void {
        trace("Field: " + name);
        for (var i = -1; i < len; ++i) {
            trace("  " + name + ".getLineIndexOfChar(" + i + ") = " + text.getLineIndexOfChar(i));
        }
    }

    private function newTextField():TextField {
        var tf:TextField = new TextField();
        tf.embedFonts = true;
        tf.defaultTextFormat = new TextFormat("Noto Sans");
        return tf;
    }
}
}
