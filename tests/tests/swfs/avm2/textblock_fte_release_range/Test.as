package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        var br:String = String.fromCharCode(0x2028);
        var tb:TextBlock = new TextBlock(new TextElement("aaa" + br + "bbb" + br + "ccc", new ElementFormat(fd, 16)));

        var a:TextLine = tb.createTextLine(null, 10000);
        var b:TextLine = tb.createTextLine(a, 10000);
        var c:TextLine = tb.createTextLine(b, 10000);

        trace("initial chain: " + countForward(tb.firstLine));
        trace("line at b begin is b: " + (tb.getTextLineAtCharIndex(b.textBlockBeginIndex) == b));

        tb.releaseLines(b, b);
        trace("released validity: " + b.validity);
        trace("released textBlock null: " + (b.textBlock == null));
        trace("a next is c: " + (a.nextLine == c));
        trace("c previous is a: " + (c.previousLine == a));
        trace("first remains a: " + (tb.firstLine == a));
        trace("last remains c: " + (tb.lastLine == c));
        trace("firstInvalid null: " + (tb.firstInvalidLine == null));
        trace("line at b begin now null: " + (tb.getTextLineAtCharIndex(b.textBlockBeginIndex) == null));
    }

    private function countForward(line:TextLine):int {
        var count:int = 0;
        while (line != null) {
            count++;
            line = line.nextLine;
        }
        return count;
    }
}
}
