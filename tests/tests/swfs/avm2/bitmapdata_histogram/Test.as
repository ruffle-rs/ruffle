package {
    import flash.display.Sprite;
    public class Test extends Sprite {
        public function Test() { }
    }
}

import flash.display.BitmapData;
import flash.geom.Rectangle;

function print(histogram: *) {
    trace("");
    trace("r " + histogram[0]);
    trace("g " + histogram[1]);
    trace("b " + histogram[2]);
    trace("a " + histogram[3]);
}

var bitmap = new BitmapData(10, 10);
print(bitmap.histogram());

bitmap.setPixel32(0, 0, 0xAABBCCDD);
print(bitmap.histogram());

bitmap.setPixel32(1, 1, 0xAABBCCDD);
print(bitmap.histogram());

bitmap.setPixel32(2, 2, 0x00BB0000);
print(bitmap.histogram());

bitmap.setPixel32(3, 3, 0x00BB0000);
print(bitmap.histogram());

bitmap.setPixel32(4, 4, 0x000000EE);
print(bitmap.histogram());

print(bitmap.histogram(new Rectangle(0, 0, 1, 1)));
print(bitmap.histogram(new Rectangle(0, 0, 2, 2)));
print(bitmap.histogram(new Rectangle(0, 0, 4, 4)));
print(bitmap.histogram(new Rectangle(1, 1, 2, 2)));
print(bitmap.histogram(new Rectangle(20, 20, 100, 100)));
print(bitmap.histogram(new Rectangle(-20, -20, 100, 100)));
