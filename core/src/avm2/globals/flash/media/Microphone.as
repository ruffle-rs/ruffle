package flash.media {
    import flash.events.EventDispatcher;

    public final class Microphone extends EventDispatcher {
        private var _codec:String = SoundCodec.NELLYMOSER;
        private var _enableVAD:Boolean = false;
        private var _encodeQuality:int = 6;
        private var _enhancedOptions:MicrophoneEnhancedOptions = new MicrophoneEnhancedOptions();
        private var _framesPerPacket:int = 2;
        private var _gain:Number = 50;
        private var _noiseSuppressionLevel:int = -30;
        private var _rate:int = 8;
        private var _silenceLevel:Number = 10;
        private var _silenceTimeout:int = 2000;
        private var _soundTransform:SoundTransform = new SoundTransform();
        private var _useEchoSuppression:Boolean = false;

        [API("672")]
        public static function getEnhancedMicrophone(index:int = -1):Microphone {
            return null;
        }

        public static function getMicrophone(index:int = -1):Microphone {
            return null;
        }

        public function setLoopBack(isLooped:Boolean = true):void {
        }

        public function setSilenceLevel(silenceLevel:Number, timeout:int = -1):void {
            this._silenceLevel = silenceLevel;
            if (timeout >= 0) {
                this._silenceTimeout = timeout;
            }
        }

        public function setUseEchoSuppression(isEchoSuppressed:Boolean):void {
            this._useEchoSuppression = isEchoSuppressed;
        }

        public function get activityLevel():Number {
            return -1;
        }

        public function get codec():String {
            return this._codec;
        }

        public function set codec(codec:String):void {
            this._codec = codec;
        }

        public function get enableVAD():Boolean {
            return this._enableVAD;
        }

        public function set enableVAD(isEnabled:Boolean):void {
            this._enableVAD = isEnabled;
        }

        public function get encodeQuality():int {
            return this._encodeQuality;
        }

        public function set encodeQuality(quality:int):void {
            this._encodeQuality = quality;
        }

        [API("672")]
        public function get enhancedOptions():MicrophoneEnhancedOptions {
            return this._enhancedOptions;
        }

        [API("672")]
        public function set enhancedOptions(params:MicrophoneEnhancedOptions):void {
            this._enhancedOptions = params;
        }

        public function get framesPerPacket():int {
            return this._framesPerPacket;
        }

        public function set framesPerPacket(fpp:int):void {
            this._framesPerPacket = fpp;
        }

        public function get gain():Number {
            return this._gain;
        }

        public function set gain(gain:Number):void {
            this._gain = gain;
        }

        public function get index():int {
            return 0;
        }

        public static function get isSupported():Boolean {
            return false;
        }

        public function get muted():Boolean {
            return true;
        }

        public function get name():String {
            return "";
        }

        public static function get names():Array {
            return [];
        }

        public function get noiseSuppressionLevel():int {
            return this._noiseSuppressionLevel;
        }

        public function set noiseSuppressionLevel(level:int):void {
            this._noiseSuppressionLevel = level;
        }

        public function get rate():int {
            return this._rate;
        }

        public function set rate(level:int):void {
            this._rate = level;
        }

        public function get silenceLevel():Number {
            return this._silenceLevel;
        }

        public function get silenceTimeout():int {
            return this._silenceTimeout;
        }

        public function get soundTransform():flash.media.SoundTransform {
            return this._soundTransform;
        }

        public function set soundTransform(tf:flash.media.SoundTransform):void {
            this._soundTransform = tf;
        }

        public function get useEchoSuppression():Boolean {
            return this._useEchoSuppression;
        }
    }
}
