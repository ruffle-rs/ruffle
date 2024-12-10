package flash.display {

    public final class GraphicsStroke implements IGraphicsStroke, IGraphicsData {
        [Ruffle(InternalSlot)]
        public var caps : String;

        [Ruffle(InternalSlot)]
        public var fill : IGraphicsFill;

        [Ruffle(InternalSlot)]
        public var joints : String;

        [Ruffle(InternalSlot)]
        public var miterLimit : Number;

        [Ruffle(InternalSlot)]
        public var pixelHinting : Boolean;

        [Ruffle(InternalSlot)]
        public var scaleMode : String;

        [Ruffle(InternalSlot)]
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
