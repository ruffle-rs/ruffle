package {
import flash.display.Sprite;
import flash.text.engine.*;

// TextBlock.createTextLine now returns a real flash.text.engine.TextLine
// display object backed by FteTextLine, not the EditText hack the
// previous build allocated. The line is null when content is null and
// non null otherwise, and the block's textLineCreationResult flips to
// success.
[SWF(width="100", height="100")]
public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var fmt:ElementFormat = new ElementFormat(fd, 14);

        var tb:TextBlock = new TextBlock(new TextElement("Hello", fmt));
        var line:TextLine = tb.createTextLine(null, 500);
        trace("line != null: " + (line != null));
        trace("result: " + tb.textLineCreationResult);
        trace("firstLine === line: " + (tb.firstLine === line));

        var tb2:TextBlock = new TextBlock();
        trace("null content: " + tb2.createTextLine(null, 500));
    }
}
}
