package flash.display {

    public final class GraphicsTrianglePath implements IGraphicsPath, IGraphicsData {
        public var culling : String;
        public var indices : Vector.<int>;
        public var uvtData : Vector.<Number>;
        public var vertices : Vector.<Number>;

        public function GraphicsTrianglePath(vertices:Vector.<Number> = null, indices:Vector.<int> = null, uvtData:Vector.<Number> = null, culling:String = "none") {
            this.culling = culling;
            this.indices = indices;
            this.uvtData = uvtData;
            this.vertices = vertices;
        }
    }

}