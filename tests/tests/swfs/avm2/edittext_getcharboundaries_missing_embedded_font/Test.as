package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

[SWF(width="400", height="400")]
public class Test extends Sprite {
    private var text:TextField;

    public function Test() {
        stage.scaleMode = "noScale";
        text = new TextField();
        text.border = true;
        text.x = 10;
        text.y = 10;
        text.width = 380;
        text.height = 380;
        text.multiline = true;
        text.embedFonts = true;
        addChild(text);

        testHtml("<font face='Unknown Font'>x y</font>");
    }

    private function testHtml(htmlText:String):void {
        text.htmlText = htmlText;
        trace("Text: " + htmlText.replace(/[\r\n]/g, "\\n"));

        testAt(-5);
        testAt(-1);
        for (var i = 0; i <= text.text.length; ++i) {
            testAt(i);
        }
    }

    private function testAt(charIndex:int):void {
        var bounds = text.getCharBoundaries(charIndex);
        trace("  text.getCharBoundaries(" + charIndex + ") = " + bounds);
    }
}
}
