package {
import flash.display.Sprite;
import flash.text.engine.*;

public class Test extends Sprite {
    // TestFont.swf is generated with the AIR SDK, as FTE needs a DefineFont4 font:
    // fontswf -4 -a TestFont -u U+0020,U+0061-0064 -o TestFont.swf TestFont.ttf
    [Embed(source="TestFont.swf", fontName="TestFont")]
    private var TestFont:Class;

    private static const BASELINES:Array = [
        TextBaseline.ROMAN,
        TextBaseline.ASCENT,
        TextBaseline.DESCENT,
        TextBaseline.IDEOGRAPHIC_TOP,
        TextBaseline.IDEOGRAPHIC_CENTER,
        TextBaseline.IDEOGRAPHIC_BOTTOM,
    ];

    public function Test() {
        for each (var size:Number in [10, 20, 40]) {
            for each (var zero:String in BASELINES) {
                var line:TextLine = lineAt("abcd", size, zero);
                var positions:Array = [];
                for each (var baseline:String in BASELINES) {
                    positions.push(baseline + "=" + line.getBaselinePosition(baseline));
                }
                trace("size=" + size + " baselineZero=" + zero + ": " + positions.join(" "));
            }
        }

        // A line keeps its baselineZero after it's released from its block.
        var released:TextLine = lineAt("abcd", 20, TextBaseline.ASCENT);
        released.textBlock.releaseLines(released, released);
        trace("released: textBlock=" + released.textBlock + " roman=" + released.getBaselinePosition(TextBaseline.ROMAN) + " descent=" + released.getBaselinePosition(TextBaseline.DESCENT));

        var invalidated:TextLine = lineAt("abcd", 20, TextBaseline.DESCENT);
        invalidated.validity = TextLineValidity.STATIC;
        trace("static: textBlock=" + invalidated.textBlock + " roman=" + invalidated.getBaselinePosition(TextBaseline.ROMAN) + " ascent=" + invalidated.getBaselinePosition(TextBaseline.ASCENT));

        var roman:TextLine = lineAt("abcd", 20, TextBaseline.ROMAN);
        for each (var invalid:String in ["", "bogus", null]) {
            try {
                trace("invalid " + invalid + ": " + roman.getBaselinePosition(invalid));
            } catch (e:Error) {
                trace("invalid " + invalid + ": " + e.name + " #" + e.errorID);
            }
        }
    }

    private function lineAt(text:String, size:Number, baselineZero:String):TextLine {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "TestFont";
        fd.fontLookup = FontLookup.EMBEDDED_CFF;

        var block:TextBlock = new TextBlock(new TextElement(text, new ElementFormat(fd, size)));
        block.baselineZero = baselineZero;
        return block.createTextLine(null, 10000);
    }
}
}
