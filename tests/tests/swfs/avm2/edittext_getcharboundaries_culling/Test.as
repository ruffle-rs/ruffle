package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false")]
    private var TestFont:Class;

    private var text:TextField;

    public function Test() {
        stage.scaleMode = "noScale";
        text = new TextField();
        text.border = true;
        text.x = 10;
        text.y = 10;
        text.width = 40;
        text.height = 4;
        text.multiline = true;
        text.embedFonts = true;
        var tf = new TextFormat();
        tf.font = "TestFont";
        tf.size = 10;
        text.defaultTextFormat = tf;
        addChild(text);

        text.text = "aaaaaaaaa\n";
        text.text += "aaaaaaaaa\n";
        text.text += "aaaaaaaaa\n";
        text.text += "aaaaaaaaa\n";
        text.text += "aaaaaaaaa\n";
        text.text += "aaaaaaaaa\n";
        text.text += "aaaaaaaaa\n";
        text.text += "aaaaaaaaa\n";
        text.text += "aaaaaaaaa\n";
        text.text += "aaaaaaaaa\n";

        testScroll(0);
        testScroll(1);
        testScroll(2);
        testScroll(3);
        testScroll(4);
    }

    private function testScroll(scroll:int):void {
        text.scrollV = scroll;
        trace("Scroll: " + scroll);
        testHeights();
    }

    private function testHeights():void {
        for (var h = 0; h < 60; ++h) {
            text.height = h;

            var nonNullLines = "";
            for (var line = 0; line < 11; line++) {
                if (testAt(line * 10 + 2)) {
                    nonNullLines += line + ",";
                }
            }
            trace(" Height: " + h + " -> " + nonNullLines);
        }
    }

    private function testAt(charIndex:int):Boolean {
        var bounds = text.getCharBoundaries(charIndex);
        return bounds != null;
    }
}
}
