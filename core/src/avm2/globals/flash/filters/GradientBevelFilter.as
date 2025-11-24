package flash.filters {
    public final class GradientBevelFilter extends BitmapFilter {
        // NOTE if reordering these fields, make sure to use the same order in
        // GradientGlowFilter; filter code assumes the slot layouts are identical

        [Ruffle(NativeAccessible)]
        private var _alphas:Array;

        [Ruffle(NativeAccessible)]
        private var _angle:Number;

        [Ruffle(NativeAccessible)]
        private var _blurX:Number;

        [Ruffle(NativeAccessible)]
        private var _blurY:Number;

        [Ruffle(NativeAccessible)]
        private var _colors:Array;

        [Ruffle(NativeAccessible)]
        private var _distance:Number;

        [Ruffle(NativeAccessible)]
        private var _knockout:Boolean;

        [Ruffle(NativeAccessible)]
        private var _quality:int;

        [Ruffle(NativeAccessible)]
        private var _ratios:Array;

        [Ruffle(NativeAccessible)]
        private var _strength:Number;

        [Ruffle(NativeAccessible)]
        private var _type:String;

        public function get alphas():Array {
            return this._alphas;
        }
        public function set alphas(value:Array):void {
            this._alphas = value;
        }

        public function get angle():Number {
            return this._angle;
        }
        public function set angle(value:Number):void {
            this._angle = value;
        }

        public function get blurX():Number {
            return this._blurX;
        }
        public function set blurX(value:Number):void {
            this._blurX = value;
        }

        public function get blurY():Number {
            return this._blurY;
        }
        public function set blurY(value:Number):void {
            this._blurY = value;
        }

        public function get colors():Array {
            return this._colors;
        }
        public function set colors(value:Array):void {
            this._colors = value;
        }

        public function get distance():Number {
            return this._distance;
        }
        public function set distance(value:Number):void {
            this._distance = value;
        }

        public function get knockout():Boolean {
            return this._knockout;
        }
        public function set knockout(value:Boolean):void {
            this._knockout = value;
        }

        public function get quality():int {
            return this._quality;
        }
        public function set quality(value:int):void {
            this._quality = value;
        }

        public function get ratios():Array {
            return this._ratios;
        }
        public function set ratios(value:Array):void {
            this._ratios = value;
        }

        public function get strength():Number {
            return this._strength;
        }
        public function set strength(value:Number):void {
            this._strength = value;
        }

        public function get type():String {
            return this._type;
        }
        public function set type(value:String):void {
            this._type = value;
        }

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

        override public function clone():BitmapFilter {
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
