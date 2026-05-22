package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// Trailing whitespace. flash.text.engine excludes trailing spaces from
// TextLine.textWidth (they overhang the line) but still counts them in
// rawTextLength/atomCount. Each sample is drawn between two markers so the
// trailing overhang is visible.
public class Test extends Sprite {
    private var penY:Number = 6;

    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        tw("no spaces", "Word");
        tw("one trailing", "Word ");
        tw("three trailing", "Word   ");
        tw("leading kept", "  Word");
        tw("leading+trailing", "  Word  ");
        tw("spaces only", "    ");
    }

    private function tw(label:String, txt:String):void {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var ef:ElementFormat = new ElementFormat(fd, 16);
        var tb:TextBlock = new TextBlock(new TextElement(txt, ef));
        var tl:TextLine = tb.createTextLine(null, 10000);
        trace(label
            + ": rawTextLength=" + tl.rawTextLength
            + " atomCount=" + tl.atomCount
            + " textWidth=" + tl.textWidth.toFixed(2)
            + " textHeight=" + tl.textHeight.toFixed(2));

        // Draw a vertical marker, the line, then a marker at textWidth so a
        // trailing-space overhang past textWidth is visible.
        var x0:Number = 60;
        graphics.lineStyle(1, 0xcc0000);
        graphics.moveTo(x0, penY);
        graphics.lineTo(x0, penY + tl.ascent + tl.descent);
        graphics.moveTo(x0 + tl.textWidth, penY);
        graphics.lineTo(x0 + tl.textWidth, penY + tl.ascent + tl.descent);
        graphics.lineStyle();
        tl.x = x0;
        tl.y = penY + tl.ascent;
        addChild(tl);
        penY += tl.ascent + tl.descent + 12;
    }
}
}
