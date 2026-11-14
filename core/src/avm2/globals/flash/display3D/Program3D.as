package flash.display3D {
    import flash.utils.ByteArray;

    [API("674")]
    [Ruffle(Abstract)]
    public final class Program3D {
        public native function upload(vertexProgram:ByteArray, fragmentProgram:ByteArray):void;

        public function dispose():void {
        }
    }
}
