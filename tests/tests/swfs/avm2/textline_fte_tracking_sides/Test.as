package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

public class Test extends Sprite {
    public function Test() {
        var normal:TextLine = lineWithTracking(0, 0);
        var leftOnly:TextLine = lineWithTracking(4, 0);
        var rightOnly:TextLine = lineWithTracking(0, 4);
        var both:TextLine = lineWithTracking(2, 2);

        trace("left tracking changes width: " + (leftOnly.textWidth > normal.textWidth));
        trace("right tracking changes width: " + (rightOnly.textWidth > normal.textWidth));
        trace("split tracking matches sum: " + near(both.textWidth, leftOnly.textWidth));
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
        return Math.abs(a - b) < 0.01;
    }
}
}
