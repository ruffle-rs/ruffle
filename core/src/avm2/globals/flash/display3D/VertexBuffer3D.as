package flash.display3D {
    import __ruffle__.stub_method;
    import flash.utils.ByteArray;
    
    [Ruffle(InstanceAllocator)]
    [API("674")]
    public final class VertexBuffer3D {
        public native function uploadFromByteArray(data:ByteArray, byteArrayOffset:int, startVertex:int, numVertices:int):void
        public native function uploadFromVector(data:Vector.<Number>, startVertex:int, numVertices:int):void

        public function dispose():void {
            stub_method("flash.display3D.VertexBuffer3D", "dispose");
        }
    }
}