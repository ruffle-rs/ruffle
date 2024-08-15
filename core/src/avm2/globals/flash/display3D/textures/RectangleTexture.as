package flash.display3D.textures {	
    import flash.display.BitmapData;
    import flash.utils.ByteArray;
    
    public final class RectangleTexture extends TextureBase {
        [API("690")]
        public native function uploadFromBitmapData(source:BitmapData):void;
        [API("690")]
        public native function uploadFromByteArray(data:ByteArray, byteArrayOffset:uint):void;
    }
}