package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

[SWF(width="200", height="50")]
public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0062")]
    private var testFont:Class;

    public function Test() {
        stage.scaleMode = "noScale";
        var mixed:TextField = new TextField();
        mixed.border = true;
        mixed.width = 199;
        mixed.embedFonts = true;
        var tf = new TextFormat();
        tf.font = "TestFont";
        tf.size = 10;
        mixed.defaultTextFormat = tf;
        mixed.height = 49;
        mixed.text = "abababababab";
        var tf = new TextFormat();
        tf.size = 15;
        mixed.setTextFormat(tf, 2, 4);
        tf.size = 20;
        mixed.setTextFormat(tf, 4, 6);
        tf.size = 5;
        mixed.setTextFormat(tf, 6, 8);
        tf.size = 30;
        mixed.setTextFormat(tf, 8, 10);
        tf.size = 40;
        mixed.setTextFormat(tf, 10, 12);
        addChild(mixed);
    }
}
}
