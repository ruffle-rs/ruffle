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
        var normal:TextLine = lineWithTracking(0, 0);
        var left:TextLine = lineWithTracking(4, 0);
        var right:TextLine = lineWithTracking(0, 4);
        var both:TextLine = lineWithTracking(4, 4);

        trace("left delta trimmed: " + near(left.textWidth - normal.textWidth, 8));
        trace("right delta trimmed: " + near(right.textWidth - normal.textWidth, 8));
        trace("both delta trimmed: " + near(both.textWidth - normal.textWidth, 16));
        trace("first atom starts at zero: " + near(left.getAtomBounds(0).x, 0));
        var last:Rectangle = right.getAtomBounds(right.atomCount - 1);
        trace("last atom ends at textWidth: " + near(last.x + last.width, right.textWidth));
    }

    private function lineWithTracking(left:Number, right:Number):TextLine {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        var ef:ElementFormat = new ElementFormat(fd, 24);
        ef.trackingLeft = left;
        ef.trackingRight = right;
        return new TextBlock(new TextElement("ABC", ef)).createTextLine(null, 400);
    }

    private function near(a:Number, b:Number):Boolean {
        return Math.abs(a - b) < 0.05;
    }
}
}
