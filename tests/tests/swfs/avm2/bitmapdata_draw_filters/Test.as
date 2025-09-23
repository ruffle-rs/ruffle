package {
import flash.display.*;
import flash.geom.*;
import flash.filters.*;
import flash.utils.*;

[SWF(width="200", height="100")]
public class Test extends MovieClip {
    public function Test() {
        var s:Sprite = new Sprite();
        s.x = 20;
        s.y = 20;
        s.graphics.beginFill(0xFF00FF);
        s.graphics.drawRect(0, 0, 60, 60);
        s.graphics.endFill();
        var matrix:Array = new Array();
        matrix = matrix.concat([1, 0, 0, 0, 0]); // red
        matrix = matrix.concat([0, 0, 1, 0, 0]); // green
        matrix = matrix.concat([0, 0, 0, 0, 0]); // blue
        matrix = matrix.concat([0, 0, 0, 1, 0]); // alpha
        s.filters = [new ColorMatrixFilter(matrix)];

        var bd:BitmapData = new BitmapData(100, 100);
        bd.fillRect(new Rectangle(0,0,100,100), 0xFF000000);
        bd.draw(s);
        var b:Bitmap = new Bitmap(bd);
        addChild(b);

        var container:Sprite = new Sprite();
        container.addChild(s);
        container.x = 100;
        addChild(container);
    }
}
}
