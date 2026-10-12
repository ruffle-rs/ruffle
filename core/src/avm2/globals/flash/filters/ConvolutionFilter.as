package flash.filters {
    public class ConvolutionFilter extends BitmapFilter {
        [Ruffle(NativeAccessible)]
        public var alpha:Number;

        [Ruffle(NativeAccessible)]
        public var bias:Number;

        [Ruffle(NativeAccessible)]
        public var clamp:Boolean;

        [Ruffle(NativeAccessible)]
        public var color:uint;

        [Ruffle(NativeAccessible)]
        public var divisor:Number;

        [Ruffle(NativeAccessible)]
        public var matrix:Array;

        [Ruffle(NativeAccessible)]
        private var _matrixX:Number = 0;

        [Ruffle(NativeAccessible)]
        private var _matrixY:Number = 0;

        [Ruffle(NativeAccessible)]
        public var preserveAlpha:Boolean;

        // Flash Player clamps the matrix dimensions to 0..15 at the property
        // boundary: assigning 16/31/255 reads back as 15, and the renderer
        // only ever sees the top-left 15x15 sub-matrix.
        public function get matrixX():Number {
            return this._matrixX;
        }

        public function set matrixX(value:Number):void {
            this._matrixX = value < 0 ? 0 : (value > 15 ? 15 : value);
        }

        public function get matrixY():Number {
            return this._matrixY;
        }

        public function set matrixY(value:Number):void {
            this._matrixY = value < 0 ? 0 : (value > 15 ? 15 : value);
        }

        public function ConvolutionFilter(
            matrixX:Number = 0,
            matrixY:Number = 0,
            matrix:Array = null,
            divisor:Number = 1.0,
            bias:Number = 0.0,
            preserveAlpha:Boolean = true,
            clamp:Boolean = true,
            color:uint = 0,
            alpha:Number = 0.0
        ) {
            this.alpha = alpha;
            this.bias = bias;
            this.clamp = clamp;
            this.color = color;
            this.divisor = divisor;
            this.matrix = matrix;
            this.matrixX = matrixX;
            this.matrixY = matrixY;
            this.preserveAlpha = preserveAlpha;
        }

        override public function clone():BitmapFilter {
            return new ConvolutionFilter(
                this.matrixX,
                this.matrixY,
                this.matrix,
                this.divisor,
                this.bias,
                this.preserveAlpha,
                this.clamp,
                this.color,
                this.alpha
            );
        }
    }
}
