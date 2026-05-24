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
        var fmt:ElementFormat = new ElementFormat(fd, 20);

        var block:TextBlock = new TextBlock(new TextElement("done", fmt));
        var line:TextLine = block.createTextLine(null, 1000);
        var complete:TextLine = block.recreateTextLine(line, line, 1000);
        trace("complete returns same: " + (complete === line));
        trace("complete result: " + block.textLineCreationResult);
        trace("complete raw unchanged: " + line.rawTextLength);

        var empty:TextBlock = new TextBlock();
        var emptyResult:TextLine = empty.recreateTextLine(line, null, 1000);
        trace("null content returns same: " + (emptyResult === line));
    }
}
}
