package flash.net {
    public final class NetStreamInfo {
        private var _audioBufferByteLength: Number;
        private var _audioBufferLength: Number;
        private var _audioByteCount: Number;
        private var _audioBytesPerSecond: Number;
        private var _audioLossRate: Number;
        private var _byteCount: Number;
        private var _currentBytesPerSecond: Number;
        private var _dataBufferByteLength: Number;
        private var _dataBufferLength: Number;
        private var _dataByteCount: Number;
        private var _dataBytesPerSecond: Number;
        private var _droppedFrames: Number;
        private var _isLive: Boolean;
        private var _maxBytesPerSecond: Number;
        private var _metaData: Object;
        private var _playbackBytesPerSecond: Number;
        private var _resourceName: String;
        private var _SRTT: Number;
        private var _uri: String;
        private var _videoBufferByteLength: Number;
        private var _videoBufferLength: Number;
        private var _videoByteCount: Number;
        private var _videoBytesPerSecond: Number;
        private var _videoLossRate: Number;
        private var _xmpData: Object;

        public function toString():String {
            __ruffle__.stub_method("flash.net.NetStreamInfo", "toString")
            return super.toString();
        }

        public function get audioBufferByteLength(): Number {
            return this._audioBufferByteLength;
        };
        public function get audioBufferLength(): Number {
            return this._audioBufferLength;
        };
        public function get audioByteCount(): Number {
            return this._audioByteCount;
        };
        public function get audioBytesPerSecond(): Number {
            return this._audioBytesPerSecond;
        };
        public function get audioLossRate(): Number {
            return this._audioLossRate;
        };
        public function get byteCount(): Number {
            return this._byteCount;
        };
        public function get currentBytesPerSecond(): Number {
            return this._currentBytesPerSecond;
        };
        public function get dataBufferByteLength(): Number {
            return this._dataBufferByteLength;
        };
        public function get dataBufferLength(): Number {
            return this._dataBufferLength;
        };
        public function get dataByteCount(): Number {
            return this._dataByteCount;
        };
        public function get dataBytesPerSecond(): Number {
            return this._dataBytesPerSecond;
        };
        public function get droppedFrames(): Number {
            return this._droppedFrames;
        };
        public function get isLive():Boolean {
            return this._isLive;
        };
        public function get maxBytesPerSecond(): Number {
            return this._maxBytesPerSecond;
        };
        public function get metaData(): Object {
            return this._metaData;
        };
        public function get playbackBytesPerSecond(): Number {
            return this._playbackBytesPerSecond;
        };
        public function get resourceName(): String {
            return this._resourceName;
        };
        public function get SRTT(): Number {
            return this._SRTT;
        };
        public function get uri(): String {
            return this._uri;
        };
        public function get videoBufferByteLength(): Number {
            return this._videoBufferByteLength;
        };
        public function get videoBufferLength(): Number {
            return this._videoBufferLength;
        };
        public function get videoByteCount(): Number {
            return this._videoByteCount;
        };
        public function get videoBytesPerSecond(): Number {
            return this._videoBytesPerSecond;
        };
        public function get videoLossRate(): Number {
            return this._videoLossRate;
        };
        public function get xmpData(): Object {
            return this._xmpData;
        };
    }
}