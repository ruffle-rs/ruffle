package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// Line breaking: TextBlock.createTextLine wraps a paragraph at a requested
// width. Traces which characters land on each line at several widths, and
// renders every produced line so the wrap is visible.
public class Test extends Sprite {
    private var penY:Number = 4;
    private var colX:Number = 8;

    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        var sentence:String = "The quick brown fox jumps over the lazy dog";
        wrap("sentence", sentence, 60);
        wrap("sentence", sentence, 120);
        wrap("sentence", sentence, 240);
        wrap("sentence", sentence, 10000);
        wrap("longword", "supercalifragilisticexpialidocious", 80);
    }

    private function wrap(label:String, txt:String, width:Number):void {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var ef:ElementFormat = new ElementFormat(fd, 14);
        var tb:TextBlock = new TextBlock(new TextElement(txt, ef));
        var line:TextLine = tb.createTextLine(null, width);
        var n:int = 0;
        trace(label + " width=" + width + ":");
        while (line != null) {
            var s:int = line.textBlockBeginIndex;
            var e:int = s + line.rawTextLength;
            trace("  line " + n + " [" + s + "," + e + ")"
                + " textWidth=" + line.textWidth.toFixed(1)
                + " \"" + txt.substring(s, e) + "\"");
            place(line);
            line = tb.createTextLine(line, width);
            n++;
        }
        trace("  lineCount=" + n);
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
