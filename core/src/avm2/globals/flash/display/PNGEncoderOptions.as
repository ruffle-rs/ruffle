package flash.display {
    [API("680")]
    public final class PNGEncoderOptions {
        public var fastCompression:Boolean;

        public function PNGEncoderOptions(fastCompression: Boolean = false) {
            this.fastCompression = fastCompression;
        }
    }
}
