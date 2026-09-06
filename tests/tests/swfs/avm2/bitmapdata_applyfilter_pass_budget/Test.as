package {
    import flash.display.BitmapData;
    import flash.display.Sprite;
    import flash.filters.ColorMatrixFilter;
    import flash.geom.Point;
    import flash.geom.Rectangle;

    [SWF(width="1", height="1")]
    public class Test extends Sprite {
        public function Test():void {
            var source:BitmapData = new BitmapData(1, 1, true, 0xFF336699);
            var destination:BitmapData = new BitmapData(1, 1, true, 0);
            var sourceRect:Rectangle = new Rectangle(0, 0, 1, 1);
            var destinationPoint:Point = new Point(0, 0);
            var filter:ColorMatrixFilter = new ColorMatrixFilter();
            var finalFilter:ColorMatrixFilter = new ColorMatrixFilter([
                1, 0, 0, 0, 16,
                0, 1, 0, 0, 0,
                0, 0, 1, 0, 0,
                0, 0, 0, 1, 0
            ]);

            // A color matrix filter records one render pass. The final operation
            // must be submitted separately instead of exceeding the pass budget.
            for (var i:uint = 0; i < 1024; i++) {
                destination.applyFilter(
                    source,
                    sourceRect,
                    destinationPoint,
                    filter
                );
            }
            destination.applyFilter(
                source,
                sourceRect,
                destinationPoint,
                finalFilter
            );

            trace("completed " + destination.getPixel32(0, 0).toString(16));
        }
    }
}
