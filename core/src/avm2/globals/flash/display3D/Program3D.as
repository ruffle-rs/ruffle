package flash.display3D {
    import __ruffle__.stub_method;
    import flash.utils.ByteArray;

    [API("674")]
    public final class Program3D {
        public native function upload(vertexProgram:ByteArray, fragmentProgram:ByteArray):void;

        public function dispose():void {
            stub_method("flash.display3D.Program3D", "dispose");
        }
    }
}