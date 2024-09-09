package flash.display3D {
    import __ruffle__.stub_method;
    import flash.utils.ByteArray;
    
    [Ruffle(InstanceAllocator)]
    [API("674")]
    public final class IndexBuffer3D {
        public native function uploadFromByteArray(data:ByteArray, byteArrayOffset:int, startOffset:int, count:int):void;
        public native function uploadFromVector(data:Vector.<uint>, startOffset:int, count:int):void;

        public function dispose():void {
            stub_method("flash.display3D.IndexBuffer3D", "dispose");
        }
    }
}