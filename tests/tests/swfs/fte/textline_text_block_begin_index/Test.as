package {
import flash.display.Sprite;
import flash.text.engine.*;

[SWF(width="200", height="100")]
public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        var fmt:ElementFormat = new ElementFormat(fd, 30);
        var block:TextBlock = new TextBlock(new TextElement("one two three", fmt));
        var first:TextLine = block.createTextLine(null, 60);
        var second:TextLine = block.createTextLine(first, 60);

        trace("first.rawTextLength: " + first.rawTextLength);
        trace("second.textBlockBeginIndex: " + second.textBlockBeginIndex);
        trace("second begins after first: " + (second.textBlockBeginIndex == first.rawTextLength));
    }
}
}
