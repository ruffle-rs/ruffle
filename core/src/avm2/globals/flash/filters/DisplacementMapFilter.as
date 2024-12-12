package flash.filters {
    import flash.display.BitmapData;
    import flash.geom.Point;

    public final class DisplacementMapFilter extends BitmapFilter {
        // FIXME these should all be getters/setters to match Flash

        [Ruffle(InternalSlot)]
        public var alpha: Number;

        [Ruffle(InternalSlot)]
        public var color: uint;

        [Ruffle(InternalSlot)]
        public var componentX: uint;

        [Ruffle(InternalSlot)]
        public var componentY: uint;

        [Ruffle(InternalSlot)]
        public var mapBitmap: BitmapData;

        [Ruffle(InternalSlot)]
        public var mapPoint: Point;

        [Ruffle(InternalSlot)]
        public var mode: String;

        [Ruffle(InternalSlot)]
        public var scaleX: Number;

        [Ruffle(InternalSlot)]
        public var scaleY: Number;

        public function DisplacementMapFilter(mapBitmap:BitmapData = null,
                                              mapPoint:Point = null,
                                              componentX:uint = 0,
                                              componentY:uint = 0,
                                              scaleX:Number = 0.0,
                                              scaleY:Number = 0.0,
                                              mode:String = "wrap",
                                              color:uint = 0,
                                              alpha:Number = 0.0) {
            this.mapBitmap = mapBitmap;
            this.mapPoint = mapPoint;
            this.componentX = componentX;
            this.componentY = componentY;
            this.scaleX = scaleX;
            this.scaleY = scaleY;
            this.mode = mode;
            this.color = color;
            this.alpha = alpha;
        }

        override public function clone(): BitmapFilter {
            return new DisplacementMapFilter(this.mapBitmap.clone(), this.mapPoint.clone(), this.componentX, this.componentY, this.scaleX, this.scaleY, this.mode, this.color, this.alpha);
        }
    }
}
