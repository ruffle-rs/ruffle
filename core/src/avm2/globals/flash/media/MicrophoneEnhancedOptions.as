package flash.media {
    [API("672")]
    public final class MicrophoneEnhancedOptions {
        public var echoPath:int = 128;
        public var isVoiceDetected:int = -1;
        public var mode:String = MicrophoneEnhancedMode.FULL_DUPLEX;
        public var nonLinearProcessing:Boolean = true;

        public function MicrophoneEnhancedOptions() {
        }
    }
}
