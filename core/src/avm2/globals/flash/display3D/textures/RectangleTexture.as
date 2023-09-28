package flash.display3D.textures {	
    import flash.display.BitmapData;
    import flash.utils.ByteArray;
    
    public final class RectangleTexture extends TextureBase {
        public native function uploadFromBitmapData(source:BitmapData):void;
        public native function uploadFromByteArray(data:ByteArray, byteArrayOffset:uint):void;
    }
}