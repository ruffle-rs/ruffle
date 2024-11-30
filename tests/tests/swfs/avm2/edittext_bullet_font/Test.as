package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

[SWF(width="200", height="200")]
public class Test extends Sprite {
    [Embed(source="TestFontBulletA.ttf", fontName="TestFontNoBullet", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFontNoBullet:Class;

    [Embed(source="TestFontBulletA.ttf", fontName="TestFontBulletA", embedAsCFF="false", unicodeRange="U+0061-U+0064,U+2022")]
    private var TestFontBulletA:Class;

    [Embed(source="TestFontBulletB.ttf", fontName="TestFontBulletB", embedAsCFF="false", unicodeRange="U+0061-U+0064,U+2022")]
    private var TestFontBulletB:Class;

    public function Test() {
        stage.scaleMode = "noScale";

        newTextField(5, 5, "TestFontBulletA", "<li>a</li>");
        newTextField(5, 55, "TestFontBulletA", "<font face='TestFontBulletB'><li>a</li></font>");
        newTextField(5, 105, "TestFontBulletA", "<li><font face='TestFontBulletB'>a</font></li>");
        newTextField(5, 155, "TestFontBulletA", "<li><font face='TestFontBulletB'>a</font><font face='TestFontBulletA'>a</font></li>");
        newTextField(105, 5, "TestFontBulletA", "<li><font face='TestFontBulletA'>a</font><font face='TestFontBulletB'>a</font></li>");
        newTextField(105, 55, "TestFontBulletA", "<li><font face='TestFontBulletA'></font><font face='TestFontBulletB'>a</font></li>");
        newTextField(105, 105, "TestFontBulletA", "<li><font face='TestFontNoBullet'>a</font><font face='TestFontBulletB'>a</font></li>");
        newTextField(105, 155, "TestFontBulletA", "<li><font face='Unknown Font'>a</font><font face='TestFontBulletB'>a</font></li>");
    }

    private function newTextField(x: int, y: int, defaultFont: String, htmlText: String):void {
        var text = new TextField();
        text.border = true;
        text.x = x;
        text.y = y;
        text.width = 90;
        text.height = 40;
        text.embedFonts = true;
        var tf = new TextFormat();
        tf.font = defaultFont;
        tf.size = 20;
        tf.leading = 5;
        text.defaultTextFormat = tf;
        text.htmlText = htmlText;
        addChild(text);
    }
}
}
