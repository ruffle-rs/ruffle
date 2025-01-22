package flash.display {

    public final class GraphicsTrianglePath implements IGraphicsPath, IGraphicsData {
        [Ruffle(NativeAccessible)]
        public var culling : String;

        [Ruffle(NativeAccessible)]
        public var indices : Vector.<int>;

        [Ruffle(NativeAccessible)]
        public var uvtData : Vector.<Number>;

        [Ruffle(NativeAccessible)]
        public var vertices : Vector.<Number>;

        public function GraphicsTrianglePath(vertices:Vector.<Number> = null, indices:Vector.<int> = null, uvtData:Vector.<Number> = null, culling:String = "none") {
            this.culling = culling;
            this.indices = indices;
            this.uvtData = uvtData;
            this.vertices = vertices;
        }
    }

}
