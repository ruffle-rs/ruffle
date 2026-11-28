package {

import flash.display.MovieClip;
import flash.display.Shape;

// Hairline strokes (lineStyle thickness 0) drawn with the drawing API must stay
// one pixel wide on screen, no matter how the shape containing them is scaled.
// Shapes are positioned on half pixels so a 1px line covers exactly one pixel.
[SWF(width="200", height="200", backgroundColor="#FFFFFF")]
public class Test extends MovieClip {
    public function Test() {
        // Uniform-ish upscale.
        addRect(10.5, 10.5, 5, 5, 6, 8);
        // Non-uniform upscale.
        addRect(50.5, 10.5, 10, 10, 10, 4);
        // Downscale.
        addRect(160.5, 10.5, 300, 400, 0.1, 0.1);
        // Extreme horizontal stretch, vertical shrink.
        addRect(10.5, 60.5, 10, 100, 18, 0.3);
        // Extreme upscale on both axes.
        addRect(10.5, 100.5, 2, 2, 90, 45);
    }

    private function addRect(x:Number, y:Number, width:Number, height:Number,
                             scaleX:Number, scaleY:Number):void {
        var shape:Shape = new Shape();
        shape.graphics.lineStyle(0, 0x000000);
        shape.graphics.beginFill(0xCCCCFF);
        shape.graphics.drawRect(0, 0, width, height);
        shape.graphics.endFill();
        shape.x = x;
        shape.y = y;
        shape.scaleX = scaleX;
        shape.scaleY = scaleY;
        addChild(shape);
    }
}
}
