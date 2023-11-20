package flash.display3D.textures {	
    import flash.display.BitmapData;
    import flash.events.Event;
    import flash.utils.ByteArray;
    import flash.utils.setTimeout;
    
    public final class Texture extends TextureBase {
        public native function uploadFromBitmapData(source:BitmapData, miplevel:uint = 0):void;
        public native function uploadFromByteArray(data:ByteArray, byteArrayOffset:uint, miplevel:uint = 0):void;
        public function uploadCompressedTextureFromByteArray(data:ByteArray, byteArrayOffset:uint, async:Boolean = false):void {
            if (async) {
                var self = this;
                // FIXME - actually run this in the background, with a copy of 'data'
                self.uploadCompressedTextureFromByteArrayInternal(data, byteArrayOffset);

                setTimeout(function() {
                    self.dispatchEvent(new Event("textureReady"));
                }, 0);
            } else {
                this.uploadCompressedTextureFromByteArrayInternal(data, byteArrayOffset);
            }
        }

        private native function uploadCompressedTextureFromByteArrayInternal(data:ByteArray, byteArrayOffset:uint):void
    }
}
