import flash.display.BitmapData;
import flash.geom.Rectangle;
import flash.geom.Matrix;

class Main {
    static function main(mc) {
        // Test: draw with clipRect restricts which dest pixels get updated.
        // dest is filled with opaque blue, source with opaque red.
        // Use explicit identity matrix to avoid null-matrix quirks.

        var dest:BitmapData = new BitmapData(50, 50, false, 0xFF0000FF);
        var src:BitmapData = new BitmapData(50, 50, false, 0xFFFF0000);
        var identity:Matrix = new Matrix();

        dest.draw(src, identity, null, null, new Rectangle(10, 10, 20, 20));

        // Inside clipRect (10,10,20,20): should be red (0xFFFF0000 = -10000)
        trace("inside (10,10): " + dest.getPixel32(10, 10).toString(16));
        trace("inside (29,29): " + dest.getPixel32(29, 29).toString(16));

        // Outside clipRect: should remain blue (0xFF0000FF = -ffff01)
        trace("outside (0,0): " + dest.getPixel32(0, 0).toString(16));
        trace("outside (9,9): " + dest.getPixel32(9, 9).toString(16));
        trace("outside (30,30): " + dest.getPixel32(30, 30).toString(16));
        trace("outside (49,49): " + dest.getPixel32(49, 49).toString(16));

        // Test: clipRect with translation matrix.
        // source (red) drawn shifted by (5,5), clipped to dest rect (10,10,20,20).
        // Source covers (5,5)-(54,54) in dest space; the clipRect restricts updates to (10,10)-(29,29).
        var dest2:BitmapData = new BitmapData(50, 50, false, 0xFF0000FF);
        var mat:Matrix = new Matrix();
        mat.translate(5, 5);
        dest2.draw(src, mat, null, null, new Rectangle(10, 10, 20, 20));

        trace("mat inside (10,10): " + dest2.getPixel32(10, 10).toString(16));
        trace("mat inside (29,29): " + dest2.getPixel32(29, 29).toString(16));
        trace("mat outside (0,0): " + dest2.getPixel32(0, 0).toString(16));
        trace("mat outside (9,9): " + dest2.getPixel32(9, 9).toString(16));
        trace("mat outside (30,30): " + dest2.getPixel32(30, 30).toString(16));

        // Test: clipRect that doesn't intersect source coverage.
        // source shifted by (40,40), so it only lands at (40,40)-(89,89) in dest space.
        // clipRect is (0,0,10,10) which doesn't overlap => dest unchanged.
        var dest3:BitmapData = new BitmapData(50, 50, false, 0xFF0000FF);
        var mat2:Matrix = new Matrix();
        mat2.translate(40, 40);
        dest3.draw(src, mat2, null, null, new Rectangle(0, 0, 10, 10));

        trace("no-overlap inside clip (0,0): " + dest3.getPixel32(0, 0).toString(16));
        trace("no-overlap at source (40,40): " + dest3.getPixel32(40, 40).toString(16));
    }
}
