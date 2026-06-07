package {
import flash.display.Sprite;
import flash.geom.Rectangle;
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
            new TextElement("AB", new ElementFormat(fd, 24))
        ).createTextLine(null, 400);
        var first:Rectangle = line.getAtomBounds(0);
        var second:Rectangle = line.getAtomBounds(1);

        trace("first width positive: " + (first.width > 0));
        trace("second width positive: " + (second.width > 0));
        trace("first before second: " + (first.x < second.x));
        trace("height matches: " + near(first.height, line.textHeight));
        trace("y matches ascent: " + near(first.y, -line.ascent));
    }

    private function near(a:Number, b:Number):Boolean {
        return Math.abs(a - b) < 0.01;
    }
}
}
