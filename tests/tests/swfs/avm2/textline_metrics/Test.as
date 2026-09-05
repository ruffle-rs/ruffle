package {
import flash.display.Sprite;
import flash.text.engine.*;

public class Test extends Sprite {
    [Embed(source="TestFont.ttf", fontName="TestFont", embedAsCFF="true", unicodeRange="U+0020,U+0061-U+0064")]
    private var TestFont:Class;

    private static const SIZES:Array = [10, 20, 40];

    public function Test() {
        for each (var size:Number in SIZES) {
            traceLine("a size=" + size, lineAt("a", size));
            traceLine("space size=" + size, lineAt(" ", size));
        }

        var textLine:TextLine = lineAt("a", 20);
        textLine.textBlock.recreateTextLine(textLine, null, 10000);
        traceLine("recreated a", textLine);

        var cases:Array = [
            ["tab", "\t"],
            ["line feed", "\n"],
            ["carriage return", "\r"],
            ["line separator", "\u2028"],
            ["paragraph separator", "\u2029"],
            ["carriage return + line feed", "\r\n"],
            ["space + tab", " \t"],
            ["b", "b"],
            ["c", "c"],
            ["d", "d"],
            ["ab", "ab"],
            ["abcd", "abcd"],
            ["a space d", "a d"],
        ];

        for each (var testCase:Array in cases) {
            traceLine(testCase[0], lineAt(testCase[1], 20));
        }
    }

    private function traceLine(name:String, line:TextLine):void {
        trace(name
            + " ascent=" + line.ascent
            + " descent=" + line.descent
            + " totalAscent=" + line.totalAscent
            + " totalDescent=" + line.totalDescent);
    }

    private function lineAt(text:String, size:Number):TextLine {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "TestFont";
        fd.fontLookup = FontLookup.EMBEDDED_CFF;

        var block:TextBlock = new TextBlock(new TextElement(text, new ElementFormat(fd, size)));
        return block.createTextLine(null, 10000);
    }
}
}
