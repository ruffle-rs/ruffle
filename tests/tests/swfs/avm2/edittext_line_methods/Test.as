package {
import flash.display.Sprite;
import flash.events.KeyboardEvent;
import flash.text.TextField;

public class Test extends Sprite {
    private var text1: TextField;
    private var text2: TextField;
    private var text3: TextField;
    private var text4: TextField;
    private var text5: TextField;
    private var text6: TextField;
    private var text7: TextField;
    private var text8: TextField;
    private var text9: TextField;
    private var text10: TextField;
    private var text11: TextField;
    private var text12: TextField;
    private var text13: TextField;
    private var text14: TextField;
    private var text15: TextField;

    public function Test() {
        text1 = new TextField();
        text1.text = "line 1\n line2\n\n line 4\n";
        text2 = new TextField();
        text2.text = "line 1\n line2";
        text3 = new TextField();
        text3.text = "";
        text4 = new TextField();
        text4.text = "\n";
        text5 = new TextField();
        text5.htmlText = "<p>line</p>";
        text6 = new TextField();
        text6.htmlText = "<p>line 1</p><p>line 2</p>";
        text7 = new TextField();
        text7.htmlText = "<p>line 1</p>\n<p>line 2</p>";
        text8 = new TextField();
        text8.htmlText = "<li>line 1 </li><li> line 2</li>";
        text9 = new TextField();
        text9.htmlText = "line 1<br/>line 2";
        text10 = new TextField();
        text10.htmlText = "<li>line 1 </li>\n\n<li> line 2</li>";
        text11 = new TextField();
        text11.htmlText = "<li>line 1 \n</li>\n<li> li\nne 2</li>";
        text12 = new TextField();
        text12.htmlText = "a\n\nb\r\rc\t\td\n\re\r\nf\r\n\rg\n\r\nh\n \n ";
        text13 = new TextField();
        text13.text = "a\n\nb\r\rc\t\td\n\re\r\nf\r\n\rg\n\r\nh\n \n ";
        text14 = new TextField();
        text14.wordWrap = true;
        text14.width = 50;
        text14.text = "first second\nthird";
        text15 = new TextField();
        text15.wordWrap = true;
        text15.width = 50;
        text15.htmlText = "<p>first</p> <b>second</b>\n<i>third</i>";

        addChild(text1);
        addChild(text2);
        addChild(text3);
        addChild(text4);
        addChild(text5);
        addChild(text6);
        addChild(text7);
        addChild(text8);
        addChild(text9);
        addChild(text10);
        addChild(text11);
        addChild(text12);
        addChild(text13);
        addChild(text14);
        addChild(text15);
        printAllValues();
    }

    private function printAllValues():void {
        printLineValues("text1", text1);
        printLineValues("text2", text2);
        printLineValues("text3", text3);
        printLineValues("text4", text4);
        printLineValues("text5", text5);
        printLineValues("text6", text6);
        printLineValues("text7", text7);
        printLineValues("text8", text8);
        printLineValues("text9", text9);
        printLineValues("text10", text10);
        printLineValues("text11", text11);
        printLineValues("text12", text12);
        printLineValues("text13", text13);
        printLineValues("text14", text14);
        printLineValues("text15", text15);
    }

    private function printLineValues(name:String, text:TextField):void {
        trace("Field: " + name);
        for (var i = -1;; ++i) {
            logError(function(): void {
                trace("  " + name + ".getLineLength(" + i + ") = " + text.getLineLength(i))
            });
            logError(function(): void {
                trace("  " + name + ".getLineOffset(" + i + ") = " + text.getLineOffset(i));
            });
            if (logError(function(): void {
                trace("  " + name + ".getLineText(" + i + ") = " + escape(text.getLineText(i)));
            }) && i >= 0) {
                break;
            }
        }
    }

    private function logError(f:Function):Boolean {
        try {
            f();
        } catch (e) {
            trace("  Error: " + e);
            return true;
        }
        return false;
    }

    private function escape(s: String): String {
        // We do not want to deal with newlines in this test,
        // just verify whether they are here or not.
        return s.replace(/\r\n/g, "\\n").replace(/\r/g, "\\n").replace(/\n/g, "\\n").replace(/\t/g, "\\t");
    }
}
}
