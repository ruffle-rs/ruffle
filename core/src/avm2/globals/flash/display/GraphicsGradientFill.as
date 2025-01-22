package flash.display {

import flash.geom.Matrix;

    public final class GraphicsGradientFill implements IGraphicsFill, IGraphicsData {
        [Ruffle(NativeAccessible)]
        public var alphas : Array;

        [Ruffle(NativeAccessible)]
        public var colors : Array;

        [Ruffle(NativeAccessible)]
        public var focalPointRatio : Number;

        [Ruffle(NativeAccessible)]
        public var interpolationMethod : String;

        [Ruffle(NativeAccessible)]
        public var matrix : Matrix;

        [Ruffle(NativeAccessible)]
        public var ratios : Array;

        [Ruffle(NativeAccessible)]
        public var spreadMethod : String;

        [Ruffle(NativeAccessible)]
        public var type : String;

        public function GraphicsGradientFill(
            type:String = "linear",
            colors:Array = null,
            alphas:Array = null,
            ratios:Array = null,
            matrix:Matrix = null,
            spreadMethod:String = SpreadMethod.PAD,
            interpolationMethod:String = InterpolationMethod.RGB,
            focalPointRatio:Number = 0.0
        ) {
            this.alphas = alphas;
            this.colors = colors;
            this.focalPointRatio = focalPointRatio;
            this.interpolationMethod = interpolationMethod;
            this.matrix = matrix;
            this.ratios = ratios;
            this.spreadMethod = spreadMethod;
            this.type = type;
        }
    }
}
