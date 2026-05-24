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
        fd.fontName = "_sans";
        var line:TextLine = new TextBlock(
            new TextElement("total metrics", new ElementFormat(fd, 28))
        ).createTextLine(null, 350);

        trace("totalAscent mirrors ascent: " + near(line.totalAscent, line.ascent));
        trace("totalDescent mirrors descent: " + near(line.totalDescent, line.descent));
        trace("totalHeight sums metrics: " + near(line.totalHeight, line.ascent + line.descent));
        trace("unjustified mirrors textWidth: " + near(line.unjustifiedTextWidth, line.textWidth));
    }

    private function near(a:Number, b:Number):Boolean {
        return Math.abs(a - b) < 0.01;
    }
}
}
