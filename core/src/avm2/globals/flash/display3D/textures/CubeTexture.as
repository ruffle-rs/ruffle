package flash.display3D.textures {
    import flash.display.BitmapData;
    public final class CubeTexture extends TextureBase {
        public native function uploadFromBitmapData(source:BitmapData, side:uint, miplevel:uint = 0):void
    }
}