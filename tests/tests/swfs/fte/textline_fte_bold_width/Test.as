package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.FontWeight;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

public class Test extends Sprite {
    public function Test() {
        var regular:TextLine = lineWithWeight(FontWeight.NORMAL);
        var bold:TextLine = lineWithWeight(FontWeight.BOLD);

        trace("bold changes width: " + (Math.abs(bold.textWidth - regular.textWidth) > 1));
        trace("bold keeps atoms: " + (bold.atomCount == regular.atomCount));
        trace("bold keeps text length: " + (bold.rawTextLength == regular.rawTextLength));
    }

    private function lineWithWeight(weight:String):TextLine {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        fd.fontWeight = weight;
        return new TextBlock(
            new TextElement("Bold width", new ElementFormat(fd, 28))
        ).createTextLine(null, 10000);
    }
}
}
