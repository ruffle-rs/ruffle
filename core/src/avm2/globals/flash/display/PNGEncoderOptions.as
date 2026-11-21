package flash.display {
    [API("680")]
    public final class PNGEncoderOptions {
        [Ruffle(NativeAccessible)]
        public var fastCompression:Boolean;

        public function PNGEncoderOptions(fastCompression:Boolean = false) {
            this.fastCompression = fastCompression;
        }
    }
}
