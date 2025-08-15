// compiled with mxmlc

package {
    import flash.display.MovieClip;
    import flash.utils.ByteArray;
    import flash.utils.Endian;
    import flash.utils.CompressionAlgorithm;

    public class Test extends MovieClip {
        public function Test() {
            var ba = createByteArray();

            ba.compress();
            print("compressed", ba, false);

            ba.uncompress();
            print("uncompressed", ba, true);

            //ba.compress("lzma");
            //print("compressed (lzma)", ba, false);

            //ba.uncompress("lzma");
            //print("uncompressed (lzma)", ba, true);

            ba.compress("deflate");
            print("compressed (deflate)", ba, false);

            ba.uncompress("deflate");
            print("uncompressed (deflate)", ba, true);

            ba.deflate();
            print("deflated", ba, false);

            ba.inflate();
            print("inflated", ba, true);

            ba.compress("zlib");
            print("compressed (zlib)", ba, false);

            ba.uncompress("zlib");
            print("uncompressed (zlib)", ba, true);

            checkZlibPrefix();
        }

        function createByteArray(): ByteArray {
            var result = new ByteArray();
            for (var i = 0; i < 100; i++) {
                result.writeByte(i);
            }
            return result;
        }

        function print(name: String, ba: ByteArray, withBytes: Boolean) {
            if (ba.position == 0) {
                trace(name + " position is at 0");
            } else if (ba.length == ba.position) {
                trace(name + " position is at end");
            } else {
                trace(name + " position is in middle");
            }

            if (withBytes) {
                var bytes = [];
                ba.position = 0;
                for (var i = 0; i < ba.length; i++) {
                    bytes.push(ba.readUnsignedByte());
                }
                trace("// " + name + " bytes");
                trace(bytes);
                trace("");
            }

            trace("");
        }

        // Issue 13773: gemcraft labirynth
        // Requires zlib->base64 values to start with "eN".
        // (which means binary header [120, 218])
        function checkZlibPrefix() {
            trace("Checking zlib header:")
            var ba = createByteArray();
            ba.compress("zlib");
            ba.position = 0;
            trace(ba.readUnsignedByte());
            trace(ba.readUnsignedByte());
        }
    }
}
