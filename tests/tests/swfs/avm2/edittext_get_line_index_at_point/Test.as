package {
import flash.display.Sprite;
import flash.display.Bitmap;
import flash.display.BitmapData;
import flash.text.TextField;
import flash.text.TextFormat;
import flash.geom.Rectangle;
import flash.utils.ByteArray;

[SWF(width="400", height="400")]
public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="false", unicodeRange="U+0061-U+0064")]
    private var TestFont:Class;

    private var colors: Array = [
        0xFFFF0000,
        0xFF00FF00,
        0xFF0000FF,
        0xFFFFFF00,
        0xFF00FFFF,
        0xFFFF00FF,
        0xFFFFFFFF,
    ];

    private var text1: TextField;

    public function Test() {
        stage.scaleMode = "noScale";

        var t1: TextField = newTextField();
        t1.htmlText = "a\n<font size='30'>aa</font>\n<textformat leading='-10'>aaa</textformat>\naa";
        renderMap(t1);

        var t2: TextField = newTextField(t1);
        t2.height = 50;
        t2.htmlText = "";
        renderMap(t2);

        var t3: TextField = newTextField(t2);
        t3.height = 50;
        t3.type = "input";
        t3.htmlText = "";
        renderMap(t3);

        var t4: TextField = newTextField(t3);
        t4.type = "input";
        t4.htmlText = "a<font size='+1'>a<font size='+1'>a<font size='+1'>a<font size='+1'>a<font size='+1'>a</font></font></font></font></font>\n<font size='5'>a</font>";
        renderMap(t4);

        var t5: TextField = newTextField(null, t4);
        t5.height = 50;
        t5.htmlText = "\n";
        renderMap(t5);

        var t6: TextField = newTextField(t5, t4);
        t6.height = 50;
        t6.type = "input";
        t6.htmlText = "\n";
        renderMap(t6);

        var t7: TextField = newTextField(t6, t4);
        t7.height = 50;
        t7.htmlText = "a\n<textformat leading='-70'>aaa</textformat>\na";
        renderMap(t7);

        var t8: TextField = newTextField(t7, t4);
        t8.htmlText = "\n\na\na";
        renderMap(t8);

        var t9: TextField = newTextField(null, t8);
        t9.height = 60;
        t9.htmlText = "\n\n\n\n\n\n\n\n";
        t9.scrollV = 3;
        renderMap(t9);

        var t10: TextField = newTextField(t9, t8);
        t10.htmlText = "<textformat leading='0'>a</textformat>\n<textformat leading='5'>a</textformat>\n<textformat leading='15'>a</textformat>\na\n";
        renderMap(t10);

        var t11: TextField = newTextField(t10, t8);
        t11.htmlText = "aaaaaaaaaaaaaaaaaaaaaaaaaa";
        t11.scrollH = 50;
        trace("scrollh = " + t11.scrollH);
        trace("maxscrollh = " + t11.maxScrollH);
        renderMap(t11);
    }

    private function newTextField(lastY: TextField = null, lastX: TextField = null):TextField {
        var tf = new TextFormat();
        tf.font = "TestFont";
        tf.size = 20;
        tf.leading = 2;

        var field: TextField = new TextField();
        field.x = 10;
        field.y = 10;
        if (lastX != null) {
            field.x = lastX.x + lastX.width + 12;
        }
        if (lastY != null) {
            field.y = lastY.y + lastY.height + 12;
        }
        field.embedFonts = true;
        field.border = true;
        field.defaultTextFormat = tf;
        field.width = 100;
        field.height = 100;
        return field;
    }

    private function renderMap(field: TextField):void {
        addChild(field);
        var w = field.width + 10;
        var h = field.height + 10;
        var data:BitmapData = new BitmapData(w, h);
        var pixels: ByteArray = new ByteArray();

        for (var y = 0; y < h; ++y) {
            for (var x = 0; x < w; ++x) {
                var line = field.getLineIndexAtPoint(x - 5, y - 5);

                var color;
                if (line == -1) {
                    color = 0xFF000000;
                } else {
                    color = colors[line % colors.length];
                }
                pixels.writeUnsignedInt(color);
            }
        }

        pixels.position = 0;
        data.setPixels(new Rectangle(0, 0, w, h), pixels);
        var bitmap:Bitmap = new Bitmap(data);
        bitmap.x = field.x - 5;
        bitmap.y = field.y - 5;
        addChild(bitmap);
    }
}
}
