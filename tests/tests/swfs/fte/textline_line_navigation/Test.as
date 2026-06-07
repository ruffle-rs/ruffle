package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// TextLine chain navigation. Every result is traced for the Flash Player trace
// comparison (the real pass/fail); the stage shows a short description and a
// self-checked PASS/FAIL verdict.
public class Test extends Sprite {
    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var tb:TextBlock = new TextBlock(new TextElement(
            "one two three four five six seven eight", new ElementFormat(fd, 16)));

        var count:int = 0;
        var line:TextLine = tb.createTextLine(null, 100);
        while (line != null) {
            count++;
            line = tb.createTextLine(line, 100);
        }
        trace("lineCount=" + count);
        trace("firstLine.previousLine null=" + (tb.firstLine.previousLine == null));
        trace("lastLine.nextLine null=" + (tb.lastLine.nextLine == null));
        trace("firstLine.textBlock==tb: " + (tb.firstLine.textBlock == tb));

        var fwd:int = 0;
        var n:TextLine = tb.firstLine;
        while (n != null) { fwd++; n = n.nextLine; }
        trace("forwardWalk=" + fwd);

        var bwd:int = 0;
        var p:TextLine = tb.lastLine;
        while (p != null) { bwd++; p = p.previousLine; }
        trace("backwardWalk=" + bwd);

        var ok:Boolean = count > 1 && fwd == count && bwd == count
            && tb.firstLine.previousLine == null
            && tb.lastLine.nextLine == null
            && tb.firstLine.textBlock == tb;

        verdict("TextLine nextLine / previousLine chain", ok);
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
