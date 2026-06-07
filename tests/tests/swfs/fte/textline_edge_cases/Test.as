package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// Edge cases of TextBlock.createTextLine and recreateTextLine: kerning
// disabled, the MAX_LINE_WIDTH path, and recreating a wrapped non-first line.
public class Test extends Sprite {
    private var penY:Number = 8;

    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";

        // 1. Kerning disabled.
        var efOff:ElementFormat = new ElementFormat(fd, 20);
        efOff.kerning = "off";
        var noKern:TextLine = new TextBlock(new TextElement("AVATAR Te.", efOff))
            .createTextLine(null, 400);
        trace("kerningOff: textWidth=" + noKern.textWidth.toFixed(2));
        show(noKern);

        // 2. createTextLine at MAX_LINE_WIDTH.
        var efOn:ElementFormat = new ElementFormat(fd, 20);
        var wide:TextLine = new TextBlock(new TextElement("max line width", efOn))
            .createTextLine(null, TextLine.MAX_LINE_WIDTH);
        trace("maxWidth: atomCount=" + wide.atomCount);
        show(wide);

        // 3. recreateTextLine on a wrapped, non-first line.
        var tb:TextBlock = new TextBlock(new TextElement("alpha beta gamma delta", efOn));
        var w1:TextLine = tb.createTextLine(null, 130);
        var w2:TextLine = tb.createTextLine(w1, 130);
        var rec:TextLine = tb.recreateTextLine(w2, w1, 130);
        trace("recreate: atomCount=" + (rec == null ? "null" : String(rec.atomCount)));
        if (rec != null) {
            show(rec);
        }
    }

    private function show(tl:TextLine):void {
        tl.x = 20;
        tl.y = penY + tl.ascent;
        addChild(tl);
        penY += 30;
    }
}
}
