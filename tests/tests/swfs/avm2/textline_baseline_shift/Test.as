package {
import flash.display.Sprite;
import flash.geom.Rectangle;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// ElementFormat.baselineShift moves a run off its baseline. In Flash's Y-down
// space a positive number shifts the run DOWN and a negative number UP (the
// opposite of CSS baseline-shift); "superscript"/"subscript" are font-relative.
// Each variant is drawn against a fixed baseline rule so the shift is visible.
public class Test extends Sprite {
    private var penY:Number = 6;

    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        shift("none", 0);
        shift("+6", 6);
        shift("-6", -6);
        shift("superscript", "superscript");
        shift("subscript", "subscript");
    }

    private function shift(label:String, value:*):void {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var ef:ElementFormat = new ElementFormat(fd, 20);
        ef.baselineShift = value;
        var tb:TextBlock = new TextBlock(new TextElement("Xy " + label, ef));
        var tl:TextLine = tb.createTextLine(null, 10000);
        var b:Rectangle = tl.getAtomBounds(0);
        trace(label
            + ": atom0.top=" + b.y.toFixed(2)
            + " atom0.height=" + b.height.toFixed(2)
            + " ascent=" + tl.ascent.toFixed(2)
            + " descent=" + tl.descent.toFixed(2)
            + " textWidth=" + tl.textWidth.toFixed(2));

        var baseY:Number = penY + tl.ascent;
        graphics.lineStyle(1, 0xcc0000);
        graphics.moveTo(8, baseY);
        graphics.lineTo(260, baseY);
        graphics.lineStyle();
        tl.x = 12;
        tl.y = baseY;
        addChild(tl);
        penY += tl.ascent + tl.descent + 12;
    }
}
}
