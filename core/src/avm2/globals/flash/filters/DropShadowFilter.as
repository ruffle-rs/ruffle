package flash.filters {
    public final class DropShadowFilter extends BitmapFilter {
        [Ruffle(NativeAccessible)]
        public var alpha: Number;

        [Ruffle(NativeAccessible)]
        public var angle: Number;

        [Ruffle(NativeAccessible)]
        public var blurX: Number;

        [Ruffle(NativeAccessible)]
        public var blurY: Number;

        [Ruffle(NativeAccessible)]
        public var color: uint;

        [Ruffle(NativeAccessible)]
        public var distance: Number;

        [Ruffle(NativeAccessible)]
        public var hideObject: Boolean;

        [Ruffle(NativeAccessible)]
        public var inner: Boolean;

        [Ruffle(NativeAccessible)]
        public var knockout: Boolean;

        [Ruffle(NativeAccessible)]
        public var quality: int;

        [Ruffle(NativeAccessible)]
        public var strength: Number;

        public function DropShadowFilter(distance:Number = 4.0,
                                        angle:Number = 45,
                                        color:uint = 0,
                                        alpha:Number = 1.0,
                                        blurX:Number = 4.0,
                                        blurY:Number = 4.0,
                                        strength:Number = 1.0,
                                        quality:int = 1,
                                        inner:Boolean = false,
                                        knockout:Boolean = false,
                                        hideObject:Boolean = false)
        {
            this.alpha = alpha;
            this.angle = angle;
            this.blurX = blurX;
            this.blurY = blurY;
            this.color = color;
            this.distance = distance;
            this.hideObject = hideObject;
            this.inner = inner;
            this.knockout = knockout;
            this.quality = quality;
            this.strength = strength;
        }

        override public function clone(): BitmapFilter {
            return new DropShadowFilter(this.distance,
                                        this.angle,
                                        this.color,
                                        this.alpha,
                                        this.blurX, 
                                        this.blurY,
                                        this.strength,
                                        this.quality,
                                        this.inner,
                                        this.knockout,
                                        this.hideObject);
        }
    }
}
