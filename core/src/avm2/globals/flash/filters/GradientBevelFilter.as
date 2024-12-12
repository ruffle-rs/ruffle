package flash.filters {
    public final class GradientBevelFilter extends BitmapFilter {
        // NOTE if reordering these fields, make sure to use the same order in
        // GradientGlowFilter; filter code assumes the slot layouts are identical

        // FIXME these should all be getters/setters to match Flash
        [Ruffle(InternalSlot)]
        public var alphas : Array;

        [Ruffle(InternalSlot)]
        public var angle : Number;

        [Ruffle(InternalSlot)]
        public var blurX : Number;

        [Ruffle(InternalSlot)]
        public var blurY : Number;

        [Ruffle(InternalSlot)]
        public var colors : Array;

        [Ruffle(InternalSlot)]
        public var distance : Number;

        [Ruffle(InternalSlot)]
        public var knockout : Boolean;

        [Ruffle(InternalSlot)]
        public var quality : int;

        [Ruffle(InternalSlot)]
        public var ratios : Array;

        [Ruffle(InternalSlot)]
        public var strength : Number;

        [Ruffle(InternalSlot)]
        public var type : String;

        public function GradientBevelFilter(
            distance:Number = 4.0,
            angle:Number = 45,
            colors:Array = null,
            alphas:Array = null,
            ratios:Array = null,
            blurX:Number = 4.0,
            blurY:Number = 4.0,
            strength:Number = 1,
            quality:int = 1,
            type:String = "inner",
            knockout:Boolean = false
        ) {
            this.distance = distance;
            this.angle = angle;
            this.colors = colors;
            this.alphas = alphas;
            this.ratios = ratios;
            this.blurX = blurX;
            this.blurY = blurY;
            this.strength = strength;
            this.quality = quality;
            this.type = type;
            this.knockout = knockout;
        }

        override public function clone(): BitmapFilter {
            return new GradientBevelFilter(
                this.distance,
                this.angle,
                this.colors,
                this.alphas,
                this.ratios,
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
