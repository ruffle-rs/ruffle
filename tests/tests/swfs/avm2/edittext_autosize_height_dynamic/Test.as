package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

[SWF(width="500", height="400")]
public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    private var nextY: int = 0;
    private var nextX: int = 0;

    public function Test() {
        stage.scaleMode = "noScale";

        newTextField("left", false, "<p></p><p>abab</p><p>ab</p>");
        newTextField("left", true, "<p></p><p>abab</p><p>ab</p>");
        newTextField("center", false, "<p></p><p>abab</p><p>ab</p>");
        newTextField("center", true, "<p></p><p>abab</p><p>ab</p>");
        newTextField("right", false, "<p></p><p>abab</p><p>ab</p>");
        newTextField("right", true, "<p></p><p>abab</p><p>ab</p>");

        nextY = 0;
        nextX = 100;
        newTextField("left", false, "<p>abab</p><p>ab</p><p></p>");
        newTextField("left", true, "<p>abab</p><p>ab</p><p></p>");
        newTextField("center", false, "<p>abab</p><p>ab</p><p></p>");
        newTextField("center", true, "<p>abab</p><p>ab</p><p></p>");
        newTextField("right", false, "<p>abab</p><p>ab</p><p></p>");
        newTextField("right", true, "<p>abab</p><p>ab</p><p></p>");

        nextY = 0;
        nextX = 200;
        newTextField("left", false, "<p>abab</p>");
        newTextField("left", true, "<p>abab</p>");
        newTextField("left", false, "");
        newTextField("left", true, "");
        newTextField("left", false, "<p></p>");
        newTextField("left", true, "<p></p>");

        nextY = 0;
        nextX = 300;
        newTextField("right", false, "<p></p>");
        newTextField("right", true, "<p></p>");
        newTextField("center", false, "<p></p>");
        newTextField("center", true, "<p></p>");
        newTextField("left", false, "<p></p><p></p>");
        newTextField("left", true, "<p></p><p></p>");

        nextY = 0;
        nextX = 400;
        newTextField("left", true, "<p>abab abab abab abab abab</p>");
        newTextField("left", true, "<p>abab abab abab abab abab</p><p></p>");
        newTextField("left", true, "abab abab abab abab abab");
        newTextField("right", true, "<p>abab abab abab abab abab</p>");
        newTextField("right", true, "<p>abab abab abab abab abab</p><p></p>");
        newTextField("right", true, "abab abab abab abab abab");
    }

    private function newTextField(autosize: String, wordWrap: Boolean, htmlText: String):void {
        var text = new TextField();
        text.multiline = true;
        text.wordWrap = wordWrap;
        text.border = true;
        text.x = nextX;
        text.y = nextY;
        nextY += 62;
        text.width = 100;
        text.height = 60;
        text.embedFonts = true;
        var tf = new TextFormat();
        tf.font = "TestFont";
        tf.size = 10;
        tf.leading = 5;
        text.defaultTextFormat = tf;
        text.htmlText = htmlText;
        text.autoSize = autosize;
        trace("(w, h) = (" + text.textWidth + ", " + text.textHeight + ")");
        trace("(x, y, w, h) = (" + text.x + ", " + text.y + ", " + text.width + ", " + text.height + ")");
        addChild(text);
    }
}
}
