package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// The atom-level TextLine API. Every result is traced for the Flash Player
// trace comparison (the real pass/fail); the stage shows a short description
// and a self-checked PASS/FAIL verdict.
public class Test extends Sprite {
    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var tl:TextLine = new TextBlock(
            new TextElement("CTSS mail", new ElementFormat(fd, 24)))
            .createTextLine(null, 10000);

        trace("atomCount=" + tl.atomCount);
        var ok:Boolean = tl.atomCount == 9;
        var prev:Number = -1;
        for (var i:int = 0; i < tl.atomCount; i++) {
            trace("  atom " + i
                + " center=" + tl.getAtomCenter(i).toFixed(2)
                + " bidi=" + tl.getAtomBidiLevel(i)
                + " wordLeft=" + tl.getAtomWordBoundaryOnLeft(i)
                + " begin=" + tl.getAtomTextBlockBeginIndex(i)
                + " end=" + tl.getAtomTextBlockEndIndex(i));
            if (tl.getAtomTextBlockBeginIndex(i) != i) ok = false;
            if (tl.getAtomTextBlockEndIndex(i) != i + 1) ok = false;
            if (tl.getAtomCenter(i) <= prev) ok = false;
            prev = tl.getAtomCenter(i);
        }
        for (var c:int = 0; c < 9; c++) {
            trace("  charIndex " + c + " -> atom " + tl.getAtomIndexAtCharIndex(c));
            if (tl.getAtomIndexAtCharIndex(c) != c) ok = false;
        }
        trace("  pointAt 50 -> atom " + tl.getAtomIndexAtPoint(50, 0));
        trace("  pointAt 9999 -> atom " + tl.getAtomIndexAtPoint(9999, 0));

        verdict("TextLine atom API: count, center, bidi, indices", ok);
    }

    private function verdict(desc:String, ok:Boolean):void {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var df:ElementFormat = new ElementFormat(fd, 17);
        var d:TextLine = new TextBlock(new TextElement(desc, df))
            .createTextLine(null, 470);
        d.x = 20;
        d.y = 60 + d.ascent;
        addChild(d);
        var vf:ElementFormat = new ElementFormat(fd, 52);
        vf.color = ok ? 0x118811 : 0xCC1111;
        var v:TextLine = new TextBlock(new TextElement(ok ? "PASS" : "FAIL", vf))
            .createTextLine(null, 470);
        v.x = 20;
        v.y = 130 + v.ascent;
        addChild(v);
    }
}
}
