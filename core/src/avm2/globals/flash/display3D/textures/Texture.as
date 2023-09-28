package flash.display3D.textures {	
    import flash.display.BitmapData;
    import flash.utils.ByteArray;
    import __ruffle__.stub_method;
    
    public final class Texture extends TextureBase {
        public native function uploadFromBitmapData(source:BitmapData, miplevel:uint = 0):void;
        public native function uploadFromByteArray(data:ByteArray, byteArrayOffset:uint, miplevel:uint = 0):void;
        public native function uploadCompressedTextureFromByteArray(data:ByteArray, byteArrayOffset:uint, async:Boolean = false):void;
    }
}