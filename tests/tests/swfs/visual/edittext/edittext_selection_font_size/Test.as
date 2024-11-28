package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

[SWF(width="250", height="60")]
public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var testFont:Class;

    public function Test() {
        stage.scaleMode = "noScale";
        newTextField();
    }

    private function newTextField(): void {
        var text:TextField = new TextField();
        text.border = true;
        text.width = 247;
        text.x = 1;
        text.y = 1;
        text.multiline = true;
        text.height = 57;
        text.embedFonts = true;
        text.selectable = true;
        text.textColor = 0xFF00FF;
        text.background = true;
        text.backgroundColor = 0x00FFFF;
        var tf = new TextFormat();
        tf.font = "TestFont";
        tf.size = 10;
        text.defaultTextFormat = tf;
        text.htmlText = "ac<font size='+4'>ac<font size='+4'>ac<font size='+4'>ac<font size='+4'>ac<font size='+4'>ac</font></font></font></font></font>";
        stage.focus = text;
        text.setSelection(1, 7);
        addChild(text);
    }
}
}
