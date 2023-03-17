package flash.display {

    public final class GraphicsStroke implements IGraphicsStroke, IGraphicsData {
        public var caps : String;
        public var fill : IGraphicsFill;
        public var joints : String;
        public var miterLimit : Number;
        public var pixelHinting : Boolean;
        public var scaleMode : String;
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