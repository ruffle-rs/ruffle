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
        var third:TextLine = tb.createTextLine(second, 80);
        var consumed:int = first.rawTextLength + second.rawTextLength + third.rawTextLength;

        trace("second text starts with long word: " + (text.substr(second.textBlockBeginIndex, 5) == "super"));
        trace("second has remaining-text break: " + (second.rawTextLength > first.rawTextLength));
        trace("third advances again: " + (third.textBlockBeginIndex == first.rawTextLength + second.rawTextLength));
        trace("three lines make progress: " + (consumed > second.rawTextLength * 2));
    }
}
}
