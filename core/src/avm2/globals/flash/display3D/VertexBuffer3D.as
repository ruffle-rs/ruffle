package flash.display3D {
    import flash.utils.ByteArray;
    
    [Ruffle(InstanceAllocator)]
    public final class VertexBuffer3D {
        public native function uploadFromByteArray(data:ByteArray, byteArrayOffset:int, startVertex:int, numVertices:int):void
        public native function uploadFromVector(data:Vector.<Number>, startVertex:int, numVertices:int):void
    }
}