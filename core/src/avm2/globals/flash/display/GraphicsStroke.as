package flash.display {

    public final class GraphicsStroke implements IGraphicsStroke, IGraphicsData {
        [Ruffle(NativeAccessible)]
        public var caps : String;

        [Ruffle(NativeAccessible)]
        public var fill : IGraphicsFill;

        [Ruffle(NativeAccessible)]
        public var joints : String;

        [Ruffle(NativeAccessible)]
        public var miterLimit : Number;

        [Ruffle(NativeAccessible)]
        public var pixelHinting : Boolean;

        [Ruffle(NativeAccessible)]
        public var scaleMode : String;

        [Ruffle(NativeAccessible)]
        public var thickness : Number;

        public function GraphicsStroke(
            thickness:Number = NaN,
            pixelHinting:Boolean = false,
            scaleMode:String = "normal",
            caps:String = "none",
            joints:String = "round",
            miterLimit:Number = 3.0,
            fill:IGraphicsFill = null
        ) {
            this.thickness = thickness;
            this.pixelHinting = pixelHinting;
            this.scaleMode = scaleMode;
            this.caps = caps;
            this.joints = joints;
            this.miterLimit = miterLimit;
            this.fill = fill;
        }
    }

}
