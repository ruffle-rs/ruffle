package flash.display3D.textures {	
    import flash.display.BitmapData;
    public final class Texture extends TextureBase {
        public native function uploadFromBitmapData(source:BitmapData, miplevel:uint = 0):void
    }
}