package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// TextBlock.recreateTextLine re-lays-out a TextLine in place, reusing the same
// object. Results traced for the Flash Player comparison (the real pass/fail);
// the stage shows a short description and a self-checked PASS/FAIL verdict.
public class Test extends Sprite {
    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var tb:TextBlock = new TextBlock(
            new TextElement("Multics terminal", new ElementFormat(fd, 20)));

        var tl:TextLine = tb.createTextLine(null, 10000);
        trace("created: textWidth=" + tl.textWidth.toFixed(2)
            + " rawTextLength=" + tl.rawTextLength + " atomCount=" + tl.atomCount);
        var w1:Number = tl.textWidth;
        var len1:int = tl.rawTextLength;
        var ac1:int = tl.atomCount;

        var again:TextLine = tb.recreateTextLine(tl, null, 10000);
        trace("recreated: textWidth=" + again.textWidth.toFixed(2)
            + " rawTextLength=" + again.rawTextLength + " atomCount=" + again.atomCount);
        trace("sameObject=" + (again == tl));

        var ok:Boolean = again == tl && again.rawTextLength == len1
            && again.atomCount == ac1 && Math.abs(again.textWidth - w1) < 0.5;

        verdict("TextBlock.recreateTextLine reuses the TextLine", ok);
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
