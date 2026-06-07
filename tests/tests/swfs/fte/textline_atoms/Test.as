package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// Exercises the TextLine atom model: rawTextLength, atomCount, per-atom
// text-block indices, bidi level, word boundaries and getAtomIndexAtCharIndex.
// The created lines are added to the stage so the scene is not blank.
public class Test extends Sprite {
    private var penY:Number = 8;

    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        var ef:ElementFormat = new ElementFormat();

        trace("=== plain word, no terminator ===");
        var tb1:TextBlock = new TextBlock(new TextElement("Hello", ef));
        var l1:TextLine = tb1.createTextLine(null, 500);
        dumpLine("Hello", l1);
        trace("atomIndexAtCharIndex:");
        for (var c:int = 0; c <= 5; c++) {
            trace("  char " + c + " -> atom " + l1.getAtomIndexAtCharIndex(c));
        }

        trace("=== text with paragraph terminator ===");
        var tb2:TextBlock = new TextBlock(new TextElement("Hi ", ef));
        var l2:TextLine = tb2.createTextLine(null, 500);
        dumpLine("Hi+PS", l2);

        trace("=== two paragraphs, wrap into lines ===");
        var tb3:TextBlock = new TextBlock(new TextElement("AAA BBB ", ef));
        var p1:TextLine = tb3.createTextLine(null, 500);
        dumpLine("line1", p1);
        var p2:TextLine = tb3.createTextLine(p1, 500);
        dumpLine("line2", p2);
        var p3:TextLine = tb3.createTextLine(p2, 500);
        dumpLine("line3", p3);

        trace("=== empty paragraph ===");
        var tb4:TextBlock = new TextBlock(new TextElement(" ", ef));
        var e1:TextLine = tb4.createTextLine(null, 500);
        dumpLine("emptyPara", e1);

        showLine(l1);
        showLine(l2);
        showLine(p1);
        showLine(p2);
        showLine(p3);
        showLine(e1);
    }

    private function showLine(tl:TextLine):void {
        if (tl == null) {
            return;
        }
        tl.x = 20;
        tl.y = penY + tl.ascent;
        addChild(tl);
        penY += 26;
    }

    private function dumpLine(label:String, tl:TextLine):void {
        if (tl == null) {
            trace(label + ": null");
            return;
        }
        trace(label + ": rawTextLength=" + tl.rawTextLength
            + " atomCount=" + tl.atomCount
            + " textBlockBeginIndex=" + tl.textBlockBeginIndex
            + " hasGraphicElement=" + tl.hasGraphicElement);
        for (var i:int = 0; i < tl.atomCount; i++) {
            trace("  atom " + i
                + ": tbBegin=" + tl.getAtomTextBlockBeginIndex(i)
                + " tbEnd=" + tl.getAtomTextBlockEndIndex(i)
                + " bidi=" + tl.getAtomBidiLevel(i)
                + " wordBoundaryLeft=" + tl.getAtomWordBoundaryOnLeft(i));
        }
    }
}
}
