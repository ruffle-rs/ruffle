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
        var line:TextLine = new TextBlock(
            new TextElement("A", new ElementFormat(fd, 18))
        ).createTextLine(null, 400);

        trace("start boundary: " + line.getAtomWordBoundaryOnLeft(0));
        trace("negative boundary: " + line.getAtomWordBoundaryOnLeft(-1));
        trace("past boundary: " + line.getAtomWordBoundaryOnLeft(1));
    }
}
}
