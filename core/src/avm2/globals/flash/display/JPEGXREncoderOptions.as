package flash.display {
    [API("680")]
    public final class JPEGXREncoderOptions {
        public var quantization: uint;
        public var colorSpace: String;
        public var trimFlexBits: uint;

        public function JPEGXREncoderOptions(quantization: uint = 20, colorSpace: String = "auto", trimFlexBits: uint = 0) {
            this.quantization = quantization;
            this.colorSpace = colorSpace;
            this.trimFlexBits = trimFlexBits;
        }
    }
}
