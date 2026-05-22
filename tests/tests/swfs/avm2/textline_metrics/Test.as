package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBaseline;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// flash.text.engine.TextLine metric reporting across the three device fonts
// and a range of point sizes: ascent/descent (OS/2 typographic metrics),
// textWidth/textHeight, and getBaselinePosition. The SWF both traces the
// numbers and renders each sample so the result is visible.
public class Test extends Sprite {
    private var penY:Number = 4;
    private var colX:Number = 8;

    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        var fonts:Array = ["_sans", "_serif", "_typewriter"];
        var sizes:Array = [8, 12, 24, 40];
        for (var fi:int = 0; fi < fonts.length; fi++) {
            for (var si:int = 0; si < sizes.length; si++) {
                dump(fonts[fi], sizes[si]);
            }
        }
    }

    private function dump(fontName:String, size:Number):void {
        var fd:FontDescription = new FontDescription();
        fd.fontName = fontName;
        var ef:ElementFormat = new ElementFormat(fd, size);
        var tb:TextBlock = new TextBlock(new TextElement("Multics CTSS", ef));
        var tl:TextLine = tb.createTextLine(null, 10000);
        trace(fontName + "@" + size
            + ": ascent=" + tl.ascent.toFixed(2)
            + " descent=" + tl.descent.toFixed(2)
            + " textHeight=" + tl.textHeight.toFixed(2)
            + " textWidth=" + tl.textWidth.toFixed(2)
            + " baseline.roman=" + tl.getBaselinePosition(TextBaseline.ROMAN).toFixed(2)
            + " baseline.ascent=" + tl.getBaselinePosition(TextBaseline.ASCENT).toFixed(2)
            + " baseline.descent=" + tl.getBaselinePosition(TextBaseline.DESCENT).toFixed(2));
        place(tl);
    }

    private function place(tl:TextLine):void {
        if (tl == null) {
            return;
        }
        if (penY + tl.ascent + tl.descent > 372) {
            penY = 4;
            colX += 250;
        }
        tl.x = colX;
        tl.y = penY + tl.ascent;
        addChild(tl);
        penY += tl.ascent + tl.descent + 3;
    }
}
}
