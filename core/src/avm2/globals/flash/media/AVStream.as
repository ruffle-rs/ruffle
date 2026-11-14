package flash.media {
    import flash.events.EventDispatcher;
    import flash.geom.Rectangle;

    [API("688")]
    public class AVStream extends EventDispatcher {
        public static const HARDWARE:String = "hardware";
        // Note: This is not a typo.
        public static const SOFTWARE:String = "sofware";
        public static const UNDEFINED:String = "undefined";

        private var _source:AVSource;
        private var _abrParameters:AVABRParameters;
        private var _backBufferTime:Number = 0;
        private var _bufferTime:Number = 0;
        private var _initialBufferTime:Number = 0;
        private var _captionStyle:AVCaptionStyle;
        private var _captionsEnabled:Boolean = false;
        private var _clientLiveOffset:Number = 0;
        private var _playState:int = AVPlayState.UNINITIALIZED;
        private var _useHardwareDecoder:Boolean = true;
        private var _volume:Number = 1.0;

        public function AVStream(source:AVSource) {
            super();
            if (source == null) {
                Error.throwError(TypeError, 2007, "source");
            }
            this._source = source;
        }

        public function get abrParameters():AVABRParameters {
            return this._abrParameters;
        }
        public function set abrParameters(value:AVABRParameters):void {
            this._abrParameters = value;
        }

        public function get backBufferLength():Number {
            return 0;
        }

        public function get backBufferTime():Number {
            return this._backBufferTime;
        }
        public function set backBufferTime(value:Number):void {
            this._backBufferTime = value;
        }

        public function get bufferLength():Number {
            return 0;
        }

        public function get bufferTime():Number {
            return this._bufferTime;
        }
        public function set bufferTime(value:Number):void {
            this._bufferTime = value;
        }

        public function get captionStyle():AVCaptionStyle {
            return this._captionStyle;
        }
        public function set captionStyle(value:AVCaptionStyle):void {
            this._captionStyle = value;
        }

        public function get captionsEnabled():Boolean {
            return this._captionsEnabled;
        }
        public function set captionsEnabled(value:Boolean):void {
            this._captionsEnabled = value;
        }

        public function get clientLiveOffset():Number {
            return this._clientLiveOffset;
        }
        public function set clientLiveOffset(value:Number):void {
            this._clientLiveOffset = value;
        }

        public function get decoderType():String {
            return UNDEFINED;
        }

        public function get droppedFrames():int {
            return 0;
        }

        public function get forwardBufferLength():Number {
            return 0;
        }

        public function get frameRate():Number {
            return 0;
        }

        public function get initialBufferTime():Number {
            return this._initialBufferTime;
        }
        public function set initialBufferTime(value:Number):void {
            this._initialBufferTime = value;
        }

        public function get playState():int {
            return this._playState;
        }

        public function get renderType():String {
            return UNDEFINED;
        }

        public function get source():AVSource {
            return this._source;
        }

        public function get time():Number {
            return 0;
        }

        public function get useHardwareDecoder():Boolean {
            return this._useHardwareDecoder;
        }
        public function set useHardwareDecoder(value:Boolean):void {
            this._useHardwareDecoder = value;
        }

        public function get volume():Number {
            return this._volume;
        }
        public function set volume(value:Number):void {
            this._volume = value;
        }

        public function dispose():AVResult {
            this._playState = AVPlayState.UNINITIALIZED;
            return new AVResult(AVResult.SUCCESS);
        }

        public function fastForward(speed:Number):AVResult {
            this._playState = AVPlayState.TRICK_PLAY;
            return new AVResult(AVResult.SUCCESS);
        }

        public function getTimeline():AVTimeline {
            return null;
        }

        public function pause():AVResult {
            this._playState = AVPlayState.PAUSED;
            return new AVResult(AVResult.SUCCESS);
        }

        public function play():AVResult {
            this._playState = AVPlayState.PLAYING;
            return new AVResult(AVResult.SUCCESS);
        }

        public function resume():AVResult {
            this._playState = AVPlayState.PLAYING;
            return new AVResult(AVResult.SUCCESS);
        }

        public function rewind(speed:Number):AVResult {
            this._playState = AVPlayState.TRICK_PLAY;
            return new AVResult(AVResult.SUCCESS);
        }

        public function seek(offset:Number, inBuffer:Boolean = true):AVResult {
            return new AVResult(AVResult.SUCCESS);
        }

        public function seekToKeyFrame(offset:Number, inBuffer:Boolean = true):AVResult {
            return new AVResult(AVResult.SUCCESS);
        }

        public function seekToLivePoint():AVResult {
            return new AVResult(AVResult.SUCCESS);
        }

        public function seekToLocalTime(periodIndex:int, time:Number):AVResult {
            return new AVResult(AVResult.SUCCESS);
        }

        public function setPlaySpeed(speed:Number, reserved:Number):AVResult {
            return new AVResult(AVResult.SUCCESS);
        }

        public function step(frames:int):AVResult {
            return new AVResult(AVResult.SUCCESS);
        }
    }
}
