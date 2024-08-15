package flash.display3D.textures {
    import flash.display.BitmapData;
    import flash.events.Event;
    import flash.utils.ByteArray;
    import flash.utils.setTimeout;

    public final class Texture extends TextureBase {
        [API("674")]
        public native function uploadFromBitmapData(source:BitmapData, miplevel:uint = 0):void;
        [API("674")]
        public native function uploadFromByteArray(data:ByteArray, byteArrayOffset:uint, miplevel:uint = 0):void;
        [API("674")]
        public function uploadCompressedTextureFromByteArray(data:ByteArray, byteArrayOffset:uint, async:Boolean = false):void {
            if (async) {
                var self = this;
                var copiedData = new ByteArray();
                data.position = 0;
                data.readBytes(copiedData);

                setTimeout(function() {
                        self.uploadCompressedTextureFromByteArrayInternal(copiedData, byteArrayOffset);
                        self.dispatchEvent(new Event("textureReady"));
                    }, 0);
            }
            else {
                this.uploadCompressedTextureFromByteArrayInternal(data, byteArrayOffset);
            }
        }

        private native function uploadCompressedTextureFromByteArrayInternal(data:ByteArray, byteArrayOffset:uint):void;
    }
}
