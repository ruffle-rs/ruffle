package {
import flash.display.Sprite;
import flash.text.TextField;

public class Test extends Sprite {
    private var text1: TextField;
    private var text2: TextField;
    private var text3: TextField;

    public function Test() {
        text1 = new TextField();
        text1.text = "line 1\n line2\n\n line 4\n";
        text2 = new TextField();
        text2.htmlText = "<p>line 1 </p><br/><li>line 2 </li>\n\n<li> line 4</li>";
        text3 = new TextField();
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
}
}
