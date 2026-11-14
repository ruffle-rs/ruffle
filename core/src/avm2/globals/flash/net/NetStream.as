package flash.net {
    import flash.net.NetConnection;
    import flash.net.NetStreamPlayOptions;
    import flash.net.NetStreamInfo;
    import flash.events.EventDispatcher;
    import flash.utils.ByteArray;
    import flash.media.Microphone;
    import flash.media.Camera;
    import flash.media.SoundTransform;
    import flash.media.VideoStreamSettings;

    [Ruffle(InstanceAllocator)]
    public class NetStream extends EventDispatcher {
        public static const CONNECT_TO_FMS:String = "connectToFMS";
        public static const DIRECT_CONNECTIONS:String = "directConnections";

        private var _audioReliable:Boolean = false;
        private var _audioSampleAccess:Boolean = false;
        private var _backBufferTime:Number = 0.0;
        private var _bufferTime:Number = 0.1;
        private var _bufferTimeMax:Number = 0.0;
        private var _checkPolicyFile:Boolean = false;
        private var _dataReliable:Boolean = false;
        private var _inBufferSeek:Boolean = false;
        private var _maxPauseBufferTime:Number = 0.0;
        private var _multicastAvailabilitySendToAll:Boolean = false;
        private var _multicastAvailabilityUpdatePeriod:Number = 0.0;
        private var _multicastFetchPeriod:Number = 0.0;
        private var _multicastPushNeighborLimit:Number = 0.0;
        private var _multicastRelayMarginDuration:Number = 0.0;
        private var _multicastWindowDuration:Number = 0.0;
        private var _soundTransform:SoundTransform = new SoundTransform();
        private var _useHardwareDecoder:Boolean = true;
        private var _useJitterBuffer:Boolean = false;
        private var _videoReliable:Boolean = false;
        private var _videoSampleAccess:Boolean = false;
        private var _videoStreamSettings:VideoStreamSettings = null;

        public function NetStream(connection:NetConnection, peer:String = CONNECT_TO_FMS) {
            super();
        }

        public function appendBytes(bytes:ByteArray):void {
        }

        public function appendBytesAction(action:String):void {
        }

        public function attach(connection:NetConnection):void {
        }

        public function attachAudio(mic:Microphone):void {
        }

        public function attachCamera(cam:Camera, ms:int = -1):void {
        }

        public function close():void {
            this.pause();
        }

        [API("674")]
        public function dispose():void {
            this.close();
        }

        public native function pause():void;

        public native function play(...args):void;

        public function play2(param:NetStreamPlayOptions):void {
        }

        [API("663")]
        public function preloadEmbeddedData(param:NetStreamPlayOptions):void {
        }

        public function publish(name:String = null, type:String = null):void {
        }

        public function receiveAudio(flag:Boolean):void {
        }

        public function receiveVideo(flag:Boolean):void {
        }

        public function receiveVideoFPS(fps:Number):void {
        }

        [API("690")]
        public static function resetDRMVouchers():void {
        }

        public native function resume():void;

        public native function seek(offset:Number):void;

        public function send(handlerName:String, ...args):void {
        }

        [API("661")]
        public function setDRMAuthenticationCredentials(userName:String, password:String, type:String):void {
        }

        public function step(frames:int):void {
        }

        public native function togglePause():void;

        public function get audioReliable():Boolean {
            return this._audioReliable;
        }
        public function set audioReliable(isReliable:Boolean):void {
            this._audioReliable = isReliable;
        }

        public function get audioSampleAccess():Boolean {
            return this._audioSampleAccess;
        }
        public function set audioSampleAccess(isAccessible:Boolean):void {
            this._audioSampleAccess = isAccessible;
        }

        public function get backBufferLength():Number {
            return 0.0;
        }

        public function get backBufferTime():Number {
            return this._backBufferTime;
        }
        public function set backBufferTime(time:Number):void {
            this._backBufferTime = time;
        }

        public function get bufferLength():Number {
            return 0.0;
        }

        public function get bufferTime():Number {
            return this._bufferTime;
        }
        public function set bufferTime(time:Number):void {
            this._bufferTime = time;
        }

        public function get bufferTimeMax():Number {
            return this._bufferTimeMax;
        }
        public function set bufferTimeMax(time:Number):void {
            this._bufferTimeMax = time;
        }

        public native function get bytesLoaded():uint;

        public native function get bytesTotal():uint;

        public function get checkPolicyFile():Boolean {
            return this._checkPolicyFile;
        }
        public function set checkPolicyFile(doCheck:Boolean):void {
            this._checkPolicyFile = doCheck;
        }

        public native function get client():Object;
        public native function set client(client:Object):void;

        public function get currentFPS():Number {
            return 0.0;
        }

        public function get dataReliable():Boolean {
            return this._dataReliable;
        }
        public function set dataReliable(isReliable:Boolean):void {
            this._dataReliable = isReliable;
        }

        public function get farID():String {
            return "";
        }

        public function get farNonce():String {
            return "";
        }

        public function get inBufferSeek():Boolean {
            return this._inBufferSeek;
        }
        public function set inBufferSeek(isInBuffer:Boolean):void {
            this._inBufferSeek = isInBuffer;
        }

        public function get info():NetStreamInfo {
            return new NetStreamInfo();
        }

        public function get liveDelay():Number {
            return 0.0;
        }

        public function get maxPauseBufferTime():Number {
            return this._maxPauseBufferTime;
        }
        public function set maxPauseBufferTime(time:Number):void {
            this._maxPauseBufferTime = time;
        }

        public function get multicastAvailabilitySendToAll():Boolean {
            return this._multicastAvailabilitySendToAll;
        }
        public function set multicastAvailabilitySendToAll(toAll:Boolean):void {
            this._multicastAvailabilitySendToAll = toAll;
        }

        public function get multicastAvailabilityUpdatePeriod():Number {
            return this._multicastAvailabilityUpdatePeriod;
        }
        public function set multicastAvailabilityUpdatePeriod(period:Number):void {
            this._multicastAvailabilityUpdatePeriod = period;
        }

        public function get multicastFetchPeriod():Number {
            return this._multicastFetchPeriod;
        }
        public function set multicastFetchPeriod(period:Number):void {
            this._multicastFetchPeriod = period;
        }

        public function get multicastInfo():NetStreamMulticastInfo {
            return new NetStreamMulticastInfo();
        }

        public function get multicastPushNeighborLimit():Number {
            return this._multicastPushNeighborLimit;
        }
        public function set multicastPushNeighborLimit(limit:Number):void {
            this._multicastPushNeighborLimit = limit;
        }

        public function get multicastRelayMarginDuration():Number {
            return this._multicastRelayMarginDuration;
        }
        public function set multicastRelayMarginDuration(dur:Number):void {
            this._multicastRelayMarginDuration = dur;
        }

        public function get multicastWindowDuration():Number {
            return this._multicastWindowDuration;
        }
        public function set multicastWindowDuration(dur:Number):void {
            this._multicastWindowDuration = dur;
        }

        public function get nearNonce():String {
            return "";
        }

        public function get objectEncoding():uint {
            return ObjectEncoding.AMF3;
        }

        public function get peerStreams():Array {
            return [];
        }

        public function get soundTransform():SoundTransform {
            return this._soundTransform;
        }
        public function set soundTransform(tf:SoundTransform):void {
            this._soundTransform = tf;
        }

        public native function get time():Number;

        public function get useHardwareDecoder():Boolean {
            return this._useHardwareDecoder;
        }
        public function set useHardwareDecoder(dec:Boolean):void {
            this._useHardwareDecoder = dec;
        }

        [API("680")]
        public function get useJitterBuffer():Boolean {
            return this._useJitterBuffer;
        }
        [API("680")]
        public function set useJitterBuffer(jbuf:Boolean):void {
            this._useJitterBuffer = jbuf;
        }

        public function get videoReliable():Boolean {
            return this._videoReliable;
        }
        public function set videoReliable(isReliable:Boolean):void {
            this._videoReliable = isReliable;
        }

        public function get videoSampleAccess():Boolean {
            return this._videoSampleAccess;
        }
        public function set videoSampleAccess(isReadable:Boolean):void {
            this._videoSampleAccess = isReadable;
        }

        [API("674")]
        public function get videoStreamSettings():VideoStreamSettings {
            return this._videoStreamSettings;
        }
        [API("674")]
        public function set videoStreamSettings(settings:VideoStreamSettings):void {
            this._videoStreamSettings = settings;
        }
    }
}
