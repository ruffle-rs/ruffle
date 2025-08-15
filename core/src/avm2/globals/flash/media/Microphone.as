package flash.media {
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;
    import __ruffle__.stub_method;

    import flash.events.EventDispatcher;

    public final class Microphone extends EventDispatcher {
        [API("672")]
        public static function getEnhancedMicrophone(index:int = -1):Microphone {
            stub_method("flash.media.Microphone", "getEnhancedMicrophone");
            return new Microphone();
        }

        public static function getMicrophone(index:int = -1):Microphone {
            stub_method("flash.media.Microphone", "getMicrophone");
            return new Microphone();
        }

        public function setLoopBack(isLooped:Boolean = true):void {
            stub_method("flash.media.Microphone", "setLoopBack");
        }

        public function setSilenceLevel(silenceLevel:Number, timeout:int = -1):void {
            stub_method("flash.media.Microphone", "setSilenceLevel");
        }

        public function setUseEchoSuppression(isEchoSuppressed:Boolean):void {
            stub_method("flash.media.Microphone", "setUseEchoSuppression");
        }

        public function get activityLevel():Number {
            stub_getter("flash.media.Microphone", "activityLevel");
            return 0.0;
        }

        public function get codec():String {
            stub_getter("flash.media.Microphone", "codec");
            return "";
        }

        public function set codec(codec:String) {
            stub_setter("flash.media.Microphone", "codec");
        }

        public function get enableVAD():Boolean {
            stub_getter("flash.media.Microphone", "enableVAD");
            return false;
        }

        public function set enableVAD(isEnabled:Boolean) {
            stub_setter("flash.media.Microphone", "enableVAD");
        }

        public function get encodeQuality():int {
            stub_getter("flash.media.Microphone", "encodeQuality");
            return 0;
        }

        public function set encodeQuality(quality:int) {
            stub_setter("flash.media.Microphone", "encodeQuality");
        }

        [API("672")]
        public function get enhancedOptions():MicrophoneEnhancedOptions {
            stub_getter("flash.media.Microphone", "enhancedOptions");
            return new MicrophoneEnhancedOptions();
        }

        [API("672")]
        public function set enhancedOptions(params:MicrophoneEnhancedOptions) {
            stub_setter("flash.media.Microphone", "enhancedOptions");
        }

        public function get framesPerPacket():int {
            stub_getter("flash.media.Microphone", "framesPerPacket");
            return 0;
        }

        public function set framesPerPacket(fpp:int) {
            stub_setter("flash.media.Microphone", "framesPerPacket");
        }

        public function get gain():Number {
            stub_getter("flash.media.Microphone", "gain");
            return 0.0;
        }

        public function set gain(gain:Number) {
            stub_setter("flash.media.Microphone", "gain");
        }

        public function get index():int {
            stub_getter("flash.media.Microphone", "index");
            return 0;
        }

        public static function get isSupported():Boolean {
            stub_getter("flash.media.Microphone", "isSupported");
            return false;
        }

        public function get muted():Boolean {
            stub_getter("flash.media.Microphone", "muted");
            return true;
        }

        public function get name():String {
            stub_getter("flash.media.Microphone", "name");
            return "";
        }

        public static function get names():Array {
            stub_getter("flash.media.Microphone", "names");
            return [];
        }

        public function get noiseSuppressionLevel():int {
            stub_getter("flash.media.Microphone", "noiseSuppressionLevel");
            return 0;
        }

        public function set noiseSuppressionLevel(level:int) {
            stub_setter("flash.media.Microphone", "noiseSuppressionLevel");
        }

        public function get rate():int {
            stub_getter("flash.media.Microphone", "rate");
            return 0;
        }

        public function set rate(level:int) {
            stub_setter("flash.media.Microphone", "rate");
        }

        public function get silenceLevel():Number {
            stub_getter("flash.media.Microphone", "silenceLevel");
            return 0;
        }

        public function get silenceTimeout():int {
            stub_getter("flash.media.Microphone", "silenceTimeout");
            return 0;
        }

        public function get soundTransform():flash.media.SoundTransform {
            stub_getter("flash.media.Microphone", "soundTransform");
            return new SoundTransform();
        }

        public function set soundTransform(tf:flash.media.SoundTransform) {
            stub_setter("flash.media.Microphone", "soundTransform");
        }

        public function get useEchoSuppression():Boolean {
            stub_getter("flash.media.Microphone", "useEchoSuppression");
            return false;
        }
    }
}
