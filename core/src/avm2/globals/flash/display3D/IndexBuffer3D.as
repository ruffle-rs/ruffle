package flash.display3D {
    import flash.utils.ByteArray;
    
    [Ruffle(InstanceAllocator)]
    public final class IndexBuffer3D {
        public native function uploadFromByteArray(data:ByteArray, byteArrayOffset:int, startOffset:int, count:int):void
        public native function uploadFromVector(data:Vector.<uint>, startOffset:int, count:int):void
    }
}