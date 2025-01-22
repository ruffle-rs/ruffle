package flash.display {

import flash.geom.Matrix;

    public final class GraphicsBitmapFill implements IGraphicsFill, IGraphicsData {
        [Ruffle(NativeAccessible)]
        public var bitmapData : BitmapData;

        [Ruffle(NativeAccessible)]
        public var matrix : Matrix;

        [Ruffle(NativeAccessible)]
        public var repeat : Boolean;

        [Ruffle(NativeAccessible)]
        public var smooth : Boolean;

        public function GraphicsBitmapFill(bitmapData:BitmapData = null, matrix:Matrix = null, repeat:Boolean = true, smooth:Boolean = false) {
            this.bitmapData = bitmapData;
            this.matrix = matrix;
            this.repeat = repeat;
            this.smooth = smooth;
        }
    }
}
