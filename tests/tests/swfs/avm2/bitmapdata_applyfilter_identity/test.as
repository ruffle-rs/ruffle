package {
    import flash.display.Sprite;
    import flash.display.BitmapData;
    import flash.geom.Rectangle;
    import flash.geom.Point;
    import flash.filters.ConvolutionFilter;
    import flash.system.fscommand;

    // applyFilter with an identity 1x1 convolution: the filtered result is a
    // plain copy, so this pins how a transparent source lands on an opaque
    // (and a transparent) destination.
    public class PT06FilterCopy extends Sprite {
        private function dump(label:String, b:BitmapData):void {
            var s:String = label + ":";
            for (var x:int = 0; x < b.width; x++) {
                s += " " + b.getPixel32(x, 0).toString(16);
            }
            trace(s);
        }

        public function PT06FilterCopy() {
            var f:ConvolutionFilter = new ConvolutionFilter(1, 1, [1], 1, 0);
            var src:BitmapData = new BitmapData(4, 1, true, 0);
            src.setPixel32(0, 0, 0xFFFF0000);
            src.setPixel32(1, 0, 0x8000FF00);
            src.setPixel32(2, 0, 0x400000FF);
            src.setPixel32(3, 0, 0x00000000);
            dump("src", src);

            var opaque:BitmapData = new BitmapData(4, 1, false, 0xFF0000FF);
            opaque.applyFilter(src, src.rect, new Point(0, 0), f);
            dump("to opaque", opaque);

            var transparent:BitmapData = new BitmapData(4, 1, true, 0x33113355);
            transparent.applyFilter(src, src.rect, new Point(0, 0), f);
            dump("to transparent", transparent);

            trace("done");
            fscommand("quit");
        }
    }
}
