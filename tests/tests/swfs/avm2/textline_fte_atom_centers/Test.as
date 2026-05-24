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
            new TextElement("XYZ", new ElementFormat(fd, 24))
        ).createTextLine(null, 400);

        for (var i:int = 0; i < line.atomCount; i++) {
            var bounds:Rectangle = line.getAtomBounds(i);
            var center:Number = line.getAtomCenter(i);
            trace(i + " center in bounds: " + (bounds.x < center && center < bounds.x + bounds.width));
            trace(i + " center midpoint: " + near(center, bounds.x + bounds.width / 2));
        }
    }

    private function near(a:Number, b:Number):Boolean {
        return Math.abs(a - b) < 0.01;
    }
}
}
