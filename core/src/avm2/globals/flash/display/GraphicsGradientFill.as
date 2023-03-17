package flash.display {

import flash.geom.Matrix;

    public final class GraphicsGradientFill implements IGraphicsFill, IGraphicsData {
        public var alphas : Array;
        public var colors : Array;
        public var focalPointRatio : Number;
        public var interpolationMethod : String;
        public var matrix : Matrix;
        public var ratios : Array;
        public var spreadMethod : String;
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