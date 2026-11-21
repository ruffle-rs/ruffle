package {
import flash.display.*;
import flash.geom.*;

[SWF(width="256", height="320", backgroundColor="#000000")]
public class Test extends MovieClip {
    private static const SIZE:int = 10;
    private static const CELL:int = 16;
    private static const BASE_OFFSET:int = 2;

    private static const KINDS:Array = [
        "vector", "bitmapNever", "bitmapNeverSmooth", "bitmapAlways", "bitmapAlwaysSmooth"
    ];
    private static const CACHING:Array = ["none", "self", "parent", "movedParent"];

    // Each band is a list of [offset, growth] pairs, one per column.
    private static const BANDS:Array = [
        ["offset", [[0, 0], [0.25, 0], [0.45, 0], [0.5, 0], [0.55, 0], [0.75, 0]]],
        ["scale", [[0, 0.25], [0, 0.5], [0, 0.75], [0, 1]]],
        ["offset+scale", [[0.25, 0.25], [0.5, 0.25], [0.75, 0.25], [0.25, 0.75], [0.5, 0.75], [0.75, 0.75]]]
    ];

    public function Test() {
        var bandX:int = 0;
        for each (var band:Array in BANDS) {
            var cells:Array = band[1];
            trace("Band " + band[0] + " at x=" + bandX + ", columns [offset, growth]: " + cells.join(" "));
            for (var col:int = 0; col < cells.length; col++) {
                for (var kindIndex:int = 0; kindIndex < KINDS.length; kindIndex++) {
                    for (var cachingIndex:int = 0; cachingIndex < CACHING.length; cachingIndex++) {
                        var row:int = kindIndex * CACHING.length + cachingIndex;
                        addCell(KINDS[kindIndex], CACHING[cachingIndex], cells[col][0], cells[col][1],
                                bandX + col * CELL, row * CELL);
                    }
                }
            }
            for (kindIndex = 0; kindIndex < KINDS.length; kindIndex++) {
                drawFrame(bandX, kindIndex * CACHING.length * CELL, cells.length * CELL, CACHING.length * CELL);
            }
            bandX += cells.length * CELL;
        }

        for (kindIndex = 0; kindIndex < KINDS.length; kindIndex++) {
            for (cachingIndex = 0; cachingIndex < CACHING.length; cachingIndex++) {
                row = kindIndex * CACHING.length + cachingIndex;
                trace("Row at y=" + (row * CELL) + ": kind=" + KINDS[kindIndex] + ", cacheAsBitmap=" + CACHING[cachingIndex]);
            }
        }
    }

    private function makeContent(kind:String):DisplayObject {
        if (kind == "vector") {
            var s:Sprite = new Sprite();
            s.graphics.beginFill(0xFF0000);
            s.graphics.drawRect(0, 0, SIZE / 2, SIZE);
            s.graphics.endFill();
            s.graphics.beginFill(0x00FF00);
            s.graphics.drawRect(SIZE / 2, 0, SIZE / 2, SIZE);
            s.graphics.endFill();
            return s;
        }

        var bd:BitmapData = new BitmapData(SIZE, SIZE, false, 0xFF0000);
        bd.fillRect(new Rectangle(SIZE / 2, 0, SIZE / 2, SIZE), 0x00FF00);
        var bmp:Bitmap = new Bitmap(bd);
        bmp.pixelSnapping = (kind.indexOf("Always") >= 0) ? PixelSnapping.ALWAYS : PixelSnapping.NEVER;
        bmp.smoothing = kind.indexOf("Smooth") >= 0;
        return bmp;
    }

    private function applyTransform(target:DisplayObject, offset:Number, growth:Number):void {
        var scale:Number = (SIZE + growth) / SIZE;
        target.scaleX = scale;
        target.scaleY = scale;
        target.x = BASE_OFFSET + offset;
        target.y = BASE_OFFSET + offset;
    }

    private function addCell(kind:String, caching:String, offset:Number, growth:Number, x:int, y:int):void {
        var content:DisplayObject = makeContent(kind);
        var cell:Sprite = new Sprite();
        cell.x = x;
        cell.y = y;
        addChild(cell);

        if (caching == "none" || caching == "self") {
            content.cacheAsBitmap = caching == "self";
            applyTransform(content, offset, growth);
            cell.addChild(content);
        } else {
            // The cached parent sits at an integer position and the content
            // is transformed inside it ("parent"), or the cached parent is
            // transformed and the content sits at its origin ("movedParent").
            var holder:Sprite = new Sprite();
            holder.cacheAsBitmap = true;
            holder.addChild(content);
            applyTransform(caching == "parent" ? content : holder, offset, growth);
            cell.addChild(holder);
        }
    }

    private function drawFrame(x:int, y:int, width:int, height:int):void {
        var frame:Shape = new Shape();
        frame.graphics.beginFill(0x0000FF);
        frame.graphics.drawRect(0, 0, width, height);
        frame.graphics.drawRect(1, 1, width - 2, height - 2);
        frame.graphics.endFill();
        frame.x = x;
        frame.y = y;
        addChild(frame);
    }
}
}
