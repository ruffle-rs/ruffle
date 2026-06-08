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
        var text:String = "wrap wrap wrap";
        var block:TextBlock = new TextBlock(
            new TextElement(text, new ElementFormat(fd, 20))
        );

        var line:TextLine = block.createTextLine(null, 58);
        var beforeRaw:int = line.rawTextLength;
        var beforeWidth:Number = line.specifiedWidth;
        var reused:TextLine = block.recreateTextLine(line, null, TextLine.MAX_LINE_WIDTH);

        trace("wrapped before: " + (beforeRaw < text.length));
        trace("same object: " + (reused === line));
        trace("raw length reused: " + (line.rawTextLength == beforeRaw));
        trace("specified width reused: " + near(line.specifiedWidth, beforeWidth));
        trace("result: " + block.textLineCreationResult);
    }

    private function near(a:Number, b:Number):Boolean {
        return Math.abs(a - b) < 0.01;
    }
}
}
