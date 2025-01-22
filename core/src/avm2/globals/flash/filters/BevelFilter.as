package flash.filters {
    public final class BevelFilter extends BitmapFilter {
        // FIXME these should all be getters/setters to match Flash

        [Ruffle(NativeAccessible)]
        public var angle : Number;

        [Ruffle(NativeAccessible)]
        public var blurX : Number;

        [Ruffle(NativeAccessible)]
        public var blurY : Number;

        [Ruffle(NativeAccessible)]
        public var distance : Number;

        [Ruffle(NativeAccessible)]
        public var highlightAlpha : Number;

        [Ruffle(NativeAccessible)]
        public var highlightColor : uint;

        [Ruffle(NativeAccessible)]
        public var knockout : Boolean;

        [Ruffle(NativeAccessible)]
        public var quality : int;

        [Ruffle(NativeAccessible)]
        public var shadowAlpha : Number;

        [Ruffle(NativeAccessible)]
        public var shadowColor : uint;

        [Ruffle(NativeAccessible)]
        public var strength : Number;

        [Ruffle(NativeAccessible)]
        public var type : String;

        public function BevelFilter(
            distance:Number = 4.0,
            angle:Number = 45,
            highlightColor:uint = 0xFFFFFF,
            highlightAlpha:Number = 1.0,
            shadowColor:uint = 0x000000,
            shadowAlpha:Number = 1.0,
            blurX:Number = 4.0,
            blurY:Number = 4.0,
            strength:Number = 1,
            quality:int = 1,
            type:String = "inner",
            knockout:Boolean = false
        ) {
            this.angle = angle;
            this.blurX = blurX;
            this.blurY = blurY;
            this.distance = distance;
            this.highlightAlpha = highlightAlpha;
            this.highlightColor = highlightColor;
            this.knockout = knockout;
            this.quality = quality;
            this.shadowAlpha = shadowAlpha;
            this.shadowColor = shadowColor;
            this.strength = strength;
            this.type = type;
        }

        override public function clone(): BitmapFilter {
            return new BevelFilter(
                this.distance,
                this.angle,
                this.highlightColor,
                this.highlightAlpha,
                this.shadowColor,
                this.shadowAlpha,
                this.blurX,
                this.blurY,
                this.strength,
                this.quality,
                this.type,
                this.knockout
            );
        }
    }
}
