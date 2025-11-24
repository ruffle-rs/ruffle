package flash.display {
    import flash.geom.Matrix;

    [API("662")]
    public final class GraphicsGradientFill implements IGraphicsFill, IGraphicsData {
        [Ruffle(NativeAccessible)]
        public var alphas:Array;

        [Ruffle(NativeAccessible)]
        public var colors:Array;

        [Ruffle(NativeAccessible)]
        public var focalPointRatio:Number;

        [Ruffle(NativeAccessible)]
        public var matrix:Matrix;

        [Ruffle(NativeAccessible)]
        public var ratios:Array;

        [Ruffle(NativeAccessible)]
        private var _interpolationMethod:String;

        [Ruffle(NativeAccessible)]
        private var _spreadMethod:String;

        [Ruffle(NativeAccessible)]
        private var _type:String;

        public function GraphicsGradientFill(
            type:String = "linear",
            colors:Array = null,
            alphas:Array = null,
            ratios:Array = null,
            matrix:* = null,
            spreadMethod:* = SpreadMethod.PAD,
            interpolationMethod:String = InterpolationMethod.RGB,
            focalPointRatio:Number = 0.0
        ) {
            this.alphas = alphas;
            this.colors = colors;
            this.focalPointRatio = focalPointRatio;
            this.matrix = matrix;
            this.ratios = ratios;

            this.interpolationMethod = interpolationMethod;
            this.spreadMethod = spreadMethod;
            this.type = type;
        }

        public function get type():String {
            return this._type;
        }
        public function set type(value:String):* {
            // TODO do validation
            this._type = value;
        }

        public function get spreadMethod():String {
            return this._spreadMethod;
        }
        public function set spreadMethod(value:String):* {
            // TODO do validation
            this._spreadMethod = value;
        }

        public function get interpolationMethod():String {
            return this._interpolationMethod;
        }
        public function set interpolationMethod(value:String):* {
            // TODO do validation
            this._interpolationMethod = value;
        }
    }
}
