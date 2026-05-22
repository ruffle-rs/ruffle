package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

// Hard line breaks. A TextBlock paragraph is delimited by U+2029 (paragraph
// separator); U+2028 (line separator) forces a break within a paragraph.
// createTextLine must split on both and never carry text across one.
public class Test extends Sprite {
    private var penY:Number = 4;
    private var colX:Number = 8;
    private var ls:String = String.fromCharCode(0x2028);
    private var ps:String = String.fromCharCode(0x2029);

    public function Test() {
        graphics.beginFill(0xFFFFFF);
        graphics.drawRect(0, 0, 500, 375);
        graphics.endFill();

        split("plain", "AAA BBB");
        split("paragraph U+2029", "AAA" + ps + "BBB");
        split("line U+2028", "AAA" + ls + "BBB");
        split("trailing U+2029", "AAA" + ps);
        split("two U+2028", "A" + ls + "B" + ls + "C");
    }

    private function split(label:String, txt:String):void {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var ef:ElementFormat = new ElementFormat(fd, 14);
        var tb:TextBlock = new TextBlock(new TextElement(txt, ef));
        var line:TextLine = tb.createTextLine(null, 10000);
        var n:int = 0;
        trace(label + ":");
        while (line != null) {
            var s:int = line.textBlockBeginIndex;
            var e:int = s + line.rawTextLength;
            trace("  line " + n + " [" + s + "," + e + ")"
                + " rawTextLength=" + line.rawTextLength
                + " atomCount=" + line.atomCount);
            place(line);
            line = tb.createTextLine(line, 10000);
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
