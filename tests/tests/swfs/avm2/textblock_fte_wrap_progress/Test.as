package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

public class Test extends Sprite {
    public function Test() {
        var text:String = "A supercalifragilistic";
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        var tb:TextBlock = new TextBlock(
            new TextElement(text, new ElementFormat(fd, 18))
        );
        var first:TextLine = tb.createTextLine(null, 80);
        var second:TextLine = tb.createTextLine(first, 80);

        trace("first begins at zero: " + (first.textBlockBeginIndex == 0));
        trace("first is short word: " + (first.rawTextLength == 2));
        trace("second begins after first: " + (second.textBlockBeginIndex == first.rawTextLength));
        trace("second consumes longer run: " + (second.rawTextLength > first.rawTextLength));
    }
}
}
