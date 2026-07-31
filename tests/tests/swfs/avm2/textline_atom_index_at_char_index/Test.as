package {
import flash.display.Sprite;
import flash.text.engine.*;

public class Test extends Sprite {
    [Embed(source="NotoSans.ttf", fontName="NotoSans", embedAsCFF="true", unicodeRange="U+0066,U+0069")]
    private var NotoSans:Class;

    public function Test() {
        traceTextBlock(
            "plain text",
            new TextBlock(new TextElement("ab\ncd", new ElementFormat())),
            [-1, 0, 1, 2, 3, 4, 5]
        );
        traceTextBlock(
            "surrogate pair",
            new TextBlock(new TextElement("A\uD83D\uDE00B", new ElementFormat())),
            [-1, 0, 1, 2, 3, 4]
        );

        var ligatureFont:FontDescription = new FontDescription();
        ligatureFont.fontName = "NotoSans";
        ligatureFont.fontLookup = FontLookup.EMBEDDED_CFF;
        var ligatureFormat:ElementFormat = new ElementFormat(ligatureFont);
        ligatureFormat.ligatureLevel = LigatureLevel.EXOTIC;
        traceTextBlock(
            "ligature",
            new TextBlock(new TextElement("fi", ligatureFormat)),
            [-1, 0, 1, 2]
        );

        var elements:Vector.<ContentElement> = new <ContentElement>[
            new TextElement("ab", new ElementFormat()),
            new GraphicElement(new Sprite(), 10, 10, new ElementFormat()),
            new TextElement("cd", new ElementFormat())
        ];
        traceTextBlock(
            "graphic element",
            new TextBlock(new GroupElement(elements)),
            [-1, 0, 1, 2, 3, 4, 5]
        );
    }

    private function traceTextBlock(name:String, block:TextBlock, charIndices:Array):void {
        trace(name);

        var line:TextLine = block.createTextLine(null, 10000);
        var lineIndex:int = 0;
        while (line) {
            trace("  line " + lineIndex);

            for each (var charIndex:int in charIndices) {
                trace("    getAtomIndexAtCharIndex(" + charIndex + ")="
                    + line.getAtomIndexAtCharIndex(charIndex));
            }

            line = block.createTextLine(line, 10000);
            lineIndex++;
        }
    }
}
}
