package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;
import flash.events.MouseEvent;

[SWF(width="900", height="300")]
public class Test extends Sprite {
    [Embed(source="TestFont1.ttf", fontName="TestFont1", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont1:Class;

    [Embed(source="TestFont2.ttf", fontName="TestFont2", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont2:Class;

    [Embed(source="TestFont3.ttf", fontName="TestFont3", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont3:Class;

    [Embed(source="TestFont4.ttf", fontName="TestFont4", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont4:Class;

    [Embed(source="TestFontHighDescent.ttf", fontName="TestFontHighDescent", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFontHighDescent:Class;

    [Embed(source="TestFontNoDescent.ttf", fontName="TestFontNoDescent", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFontNoDescent:Class;

    private var nextX:int = 1;

    public function Test() {
        var tf1 = newTextField();
        addChild(tf1);

        // Various fonts with different underline parameters
        tf1.htmlText += "ac<u>ac</u> <font face='TestFont2'>ac<u>ac</u></font><font face='TestFont3'>ac<u>ac</u></font> <font face='TestFont4'>ac<u>ac</u></font>\n";

        // Different colors
        tf1.htmlText += "<u>ac<font color='#ff0000'>ac<font color='#00ff00'>ac</font></font><font color='#0000ff'>ac</font></u>\n";

        // Different leading
        tf1.htmlText += "<textformat leading='10'>ac<u>ac</u>ac</textformat>";

        // Characters with different ascent/descent
        tf1.htmlText += "a<u>a</u>";
        tf1.htmlText += "c<u>c</u>";

        // High descent
        tf1.htmlText += "<font face='TestFontHighDescent'>ac<u>ac</u></font>";

        var tf2 = newTextField();
        addChild(tf2);

        // Different font sizes
        tf2.htmlText += "<u>ac<font size='+40'>ac</font></u>\n";
        tf2.htmlText += "<font size='+40'><u>ac<font size='-40'>ac</font></u></font>\n";
        tf2.htmlText += "<u>ac</u>ac<font size='+40'><u>ac</u></font>\n";
        tf2.htmlText += "<font size='+40'><u>ac</u><font size='-40'>ac<u>ac</u></font></font>\n";

        // Precise placement, high descent
        var tf3 = newTextField(100);
        addChild(tf3);
        tf3.htmlText += "<u><font face='TestFontHighDescent' size='260'>bd</font></u>\n";

        // Precise placement, no descent
        var tf4 = newTextField(100);
        addChild(tf4);
        tf4.htmlText += "<u><font face='TestFontNoDescent' size='260'>bb</font></u>\n";
    }

    private function newTextField(width: int = 300): TextField {
        var text:TextField = new TextField();
        text.border = true;
        text.type = "input";
        text.width = width - 3;
        text.height = 300 - 3;
        text.x = this.nextX;
        text.y = 1;
        text.embedFonts = true;

        var tf:TextFormat = new TextFormat();
        tf.font = "TestFont1";
        tf.size = 20;
        tf.leading = 2;
        text.defaultTextFormat = tf;

        text.multiline = true;
        text.htmlText = "";

        this.nextX += text.width + 3;

        return text;
    }

}
}
