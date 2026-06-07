package {
import flash.display.Sprite;
import flash.text.engine.*;

[SWF(width="100", height="100")]
public class Test extends Sprite {
    public function Test() {
        var block:TextBlock = new TextBlock(new TextElement("Hello", new ElementFormat()));
        var line:TextLine = block.createTextLine(null, 321);

        trace("line.rawTextLength: " + line.rawTextLength);
        trace("line.textBlockBeginIndex: " + line.textBlockBeginIndex);
        trace("line.specifiedWidth: " + line.specifiedWidth);
        trace("line.textBlock === block: " + (line.textBlock === block));
    }
}
}
