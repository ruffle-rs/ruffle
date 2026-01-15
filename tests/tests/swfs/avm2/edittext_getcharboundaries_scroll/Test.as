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
        text.height = 40;
        text.multiline = true;
        text.embedFonts = true;
        var tf = new TextFormat();
        tf.font = "TestFont";
        tf.size = 10;
        text.defaultTextFormat = tf;
        addChild(text);

        text.text = "aaaaaaaaaaaaaaaa\n";
        text.text += "aaaaaaaaaaaaaaaa\n";
        text.text += "aaaaaaaaaaaaaaaa\n";
        text.text += "aaaaaaaaaaaaaaaa\n";
        text.text += "aaaaaaaaaaaaaaaa\n";

        text.scrollH = 20;
        text.scrollV = 2;

        for (var i = 0; i < text.length; ++i) {
            testAt(i);
        }
    }

    private function testAt(charIndex:int):void {
        var bounds = text.getCharBoundaries(charIndex);
        trace("  text.getCharBoundaries(" + charIndex + ") = " + bounds);
    }
}
}
