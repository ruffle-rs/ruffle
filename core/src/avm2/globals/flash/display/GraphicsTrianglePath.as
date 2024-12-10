package flash.display {

    public final class GraphicsTrianglePath implements IGraphicsPath, IGraphicsData {
        [Ruffle(InternalSlot)]
        public var culling : String;

        [Ruffle(InternalSlot)]
        public var indices : Vector.<int>;

        [Ruffle(InternalSlot)]
        public var uvtData : Vector.<Number>;

        [Ruffle(InternalSlot)]
        public var vertices : Vector.<Number>;

        public function GraphicsTrianglePath(vertices:Vector.<Number> = null, indices:Vector.<int> = null, uvtData:Vector.<Number> = null, culling:String = "none") {
            this.culling = culling;
            this.indices = indices;
            this.uvtData = uvtData;
            this.vertices = vertices;
        }
    }

}
