package flash.display3D.textures {
    import flash.display.BitmapData;
    import flash.utils.ByteArray;
    import __ruffle__.stub_method;
    
    public final class CubeTexture extends TextureBase {
        [API("674")]
        public native function uploadFromBitmapData(source:BitmapData, side:uint, miplevel:uint = 0):void;
        [API("674")]
        public native function uploadFromByteArray(data:ByteArray, byteArrayOffset:uint, side:uint, miplevel:uint = 0);
        [API("674")]
        public native function uploadCompressedTextureFromByteArray(data:ByteArray, byteArrayOffset:uint, async:Boolean = false):void;
    }
}