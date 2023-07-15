package flash.display {

    public final class GraphicsSolidFill implements IGraphicsFill, IGraphicsData {
        public var alpha : Number = 1.0;
        public var color : uint = 0;

        public function GraphicsSolidFill(color:uint = 0, alpha:Number = 1.0) {
            this.alpha = alpha;
            this.color = color;
        }
    }

}