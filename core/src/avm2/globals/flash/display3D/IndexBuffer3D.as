package flash.display3D {
    import flash.utils.ByteArray;

    [API("674")]
    [Ruffle(Abstract)]
    public final class IndexBuffer3D {
        public native function uploadFromByteArray(data:ByteArray, byteArrayOffset:int, startOffset:int, count:int):void;
        public native function uploadFromVector(data:Vector.<uint>, startOffset:int, count:int):void;

        public function dispose():void {
        }
    }
}
