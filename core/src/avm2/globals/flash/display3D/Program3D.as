package flash.display3D {
    import flash.utils.ByteArray;
    
    public final class Program3D {
        public native function upload(vertexProgram:ByteArray, fragmentProgram:ByteArray):void;
    }
}