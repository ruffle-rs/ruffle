package {
import flash.display.Sprite;
import flash.text.engine.*;

public class Test extends Sprite {
    public function Test() {
        trace("One line per line break");
        dump("ab\ncd");
        trace("");

        trace("Carriage Return");
        dump("ab\rcd");
        trace("");

        trace("Carriage Return + Line Feed");
        dump("ab\r\ncd");
        trace("");

        trace("Double Line Feed");
        dump("ab\n\ncd");
        trace("");

        trace("Line separator");
        dump("ab\u2028cd");
        trace("");

        trace("Paragraph separator");
        dump("ab\u2029cd");
        trace("");

        trace("No line break");
        dump("abcd");
        trace("");

        trace("Trailing line break");
        dump("ab\n");
    }

    private function dump(text:String):void {
        var block:TextBlock = new TextBlock(new TextElement(text, getElementFormat()));
        var line:TextLine = block.createTextLine(null, 10000);
        var index:int = 0;
        while (line) {
            trace("  line " + index
                + ": rawTextLength=" + line.rawTextLength
                + " textBlockBeginIndex=" + line.textBlockBeginIndex
                + " first=" + (block.firstLine == line));
            line = block.createTextLine(line, 10000);
            index++;
        }
    }

    private function getElementFormat():ElementFormat {
        return new ElementFormat();
    }
}
}
