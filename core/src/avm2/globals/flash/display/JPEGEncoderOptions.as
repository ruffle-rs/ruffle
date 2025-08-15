package flash.display {
    [API("680")]
    public final class JPEGEncoderOptions {
        public var quality: uint;

        public function JPEGEncoderOptions(quality: uint = 80) {
            this.quality = quality;
        }
    }
}
