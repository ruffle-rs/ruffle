package {
import flash.display.Sprite;
import flash.text.TextField;

public class Test extends Sprite {
    private var text:TextField;

    public function Test() {
        text = new TextField();
        text.border = true;
        text.x = 20;
        text.y = 20;
        text.width = 200;
        addChild(text);

        test("hello", false, [0, 4, 5, 6, -1]);
        test("hello\n", false, [0, 4, 5, 6, 7]);
        test("hello\nworld", false, [3, 4, 5, 6, 7, 10, 11, 12]);
        test("hello\n world\n", false, [4, 5, 6, 12, 13, 14]);
        test("hello", true, [0, 4, 5, 6]);
        test("<p>hello</p>", true, [0, 4, 5, 6, 7]);
        test("<p>a</p><p>b</p>", true, [0, 1, 2, 3, 4, 5]);
        test("<li>a</li> \n <p>b</p>", true, [0, 1, 2, 3, 4, 5, 6, 7, 8]);
        test("", false, [-2, -1, 0, 1, 2]);
        test("", true, [-2, -1, 0, 1, 2]);
    }

    private function test(value:String, html:Boolean, indices:Array):void {
        text.multiline = false;
        if (html) {
            text.htmlText = value;
            trace("HTML: " + value.replace(/[\r\n]/g, "\\n"));
        } else {
            text.text = value;
        }
        trace("Text: " + text.text.replace(/[\r\n]/g, "\\n"));
        for (var i = 0; i < indices.length; ++i) {
            testAt(indices[i]);
        }

        text.multiline = true;
        if (html) {
            text.htmlText = value;
        } else {
            text.text = value;
        }
        trace("Multiline");
        for (var i = 0; i < indices.length; ++i) {
            testAt(indices[i]);
        }
    }

    private function testAt(charIndex:int):void {
        trace("  text.getFirstCharInParagraph(" + charIndex + ") = " + text.getFirstCharInParagraph(charIndex));
        trace("  text.getParagraphLength(" + charIndex + ") = " + text.getParagraphLength(charIndex));
    }
}
}
