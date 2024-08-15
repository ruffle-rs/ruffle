package flash.net {
    import flash.net.NetConnection;
    import flash.net.NetStreamPlayOptions;
    import flash.net.NetStreamInfo;
    import flash.events.EventDispatcher;
    import flash.utils.ByteArray;
    import flash.media.Microphone;
    import flash.media.Camera;
    import flash.media.VideoStreamSettings;

    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;

    [Ruffle(InstanceAllocator)]
    public class NetStream extends EventDispatcher {
        public static const CONNECT_TO_FMS: String = "connectToFMS";
        public static const DIRECT_CONNECTIONS: String = "directConnections";

        public function NetStream(connection:NetConnection, peer:String = CONNECT_TO_FMS) {

        }

        public function appendBytes(bytes:ByteArray) {
            stub_method("flash.net.NetStream", "appendBytes");
        }

        public function appendBytesAction(action:String) {
            stub_method("flash.net.NetStream", "appendBytesAction");
        }

        public function attach(connection:NetConnection) {
            stub_method("flash.net.NetStream", "attach");
        }

        public function attachAudio(mic:Microphone) {
            stub_method("flash.net.NetStream", "attachAudio");
        }

        public function attachCamera(cam:Camera, ms:int = -1) {
            stub_method("flash.net.NetStream", "attachCamera");
        }

        public function close() {
            stub_method("flash.net.NetStream", "close");
        }

        [API("674")]
        public function dispose() {
            stub_method("flash.net.NetStream", "dispose");
        }

        public native function pause();
        
        public native function play(...args);
        
        public function play2(param:NetStreamPlayOptions) {
            stub_method("flash.net.NetStream", "play2");
        }

        [API("663")]
        public function preloadEmbeddedData(param:NetStreamPlayOptions) {
            stub_method("flash.net.NetStream", "preloadEmbeddedData");
        }

        public function publish(name:String=null, type:String=null) {
            stub_method("flash.net.NetStream", "publish");
        }

        public function receiveAudio(flag:Boolean) {
            stub_method("flash.net.NetStream", "receiveAudio");
        }

        public function receiveVideo(flag:Boolean) {
            stub_method("flash.net.NetStream", "receiveVideo");
        }

        public function receiveVideoFPS(fps:Number) {
            stub_method("flash.net.NetStream", "receiveVideoFPS");
        }

        [API("690")]
        public static function resetDRMVouchers() {
            stub_method("flash.net.NetStream", "resetDRMVouchers");
        }

        public native function resume();

        public native function seek(offset:Number);

        public function send(handlerName:String, ...args) {
            stub_method("flash.net.NetStream", "send");
        }

        [API("661")]
        public function setDRMAuthenticationCredentials(userName:String, password:String, type:String) {
            stub_method("flash.net.NetStream", "setDRMAuthenticationCredentials");
        }

        public function step(frames:int) {
            stub_method("flash.net.NetStream", "step");
        }

        public native function togglePause();

        public function get audioReliable():Boolean {
            stub_getter("flash.net.NetStream", "audioReliable");
            return false;
        }

        public function set audioReliable(isReliable:Boolean) {
            stub_setter("flash.net.NetStream", "audioReliable");
        }

        public function get audioSampleAccess():Boolean {
            stub_getter("flash.net.NetStream", "audioSampleAccess");
            return false;
        }

        public function set audioSampleAccess(isAccessible:Boolean) {
            stub_setter("flash.net.NetStream", "audioSampleAccess");
        }

        public function get backBufferLength():Number {
            stub_getter("flash.net.NetStream", "backBufferLength");
            return 0.0;
        }

        public function get backBufferTime():Number {
            stub_getter("flash.net.NetStream", "backBufferTime");
            return 0.0;
        }

        public function set backBufferTime(time:Number) {
            stub_setter("flash.net.NetStream", "backBufferTime");
        }

        public function get bufferLength():Number {
            stub_getter("flash.net.NetStream", "bufferLength");
            return 0.0;
        }

        public function get bufferTime():Number {
            stub_getter("flash.net.NetStream", "bufferTime");
            return 0.0;
        }

        public function set bufferTime(time:Number) {
            stub_setter("flash.net.NetStream", "bufferTime");
        }

        public function get bufferTimeMax():Number {
            stub_getter("flash.net.NetStream", "bufferTimeMax");
            return 0.0;
        }

        public function set bufferTimeMax(time:Number) {
            stub_setter("flash.net.NetStream", "bufferTimeMax");
        }

        public native function get bytesLoaded():uint;

        public native function get bytesTotal():uint;

        public function get checkPolicyFile():Boolean {
            stub_getter("flash.net.NetStream", "checkPolicyFile");
            return false;
        }

        public function set checkPolicyFile(doCheck:Boolean) {
            stub_setter("flash.net.NetStream", "checkPolicyFile");
        }

        public native function get client():Object;

        public native function set client(client:Object);

        public function get currentFPS():Number {
            stub_getter("flash.net.NetStream", "currentFPS");
            return 0.0;
        }

        public function get dataReliable():Boolean {
            stub_getter("flash.net.NetStream", "dataReliable");
            return false;
        }

        public function set dataReliable(isReliable:Boolean) {
            stub_setter("flash.net.NetStream", "dataReliable");
        }

        public function get farID():String {
            stub_getter("flash.net.NetStream", "farID");
            return "";
        }

        public function get farNonce():String {
            stub_getter("flash.net.NetStream", "farNonce");
            return "";
        }

        public function get inBufferSeek():Boolean {
            stub_getter("flash.net.NetStream", "inBufferSeek");
            return false;
        }

        public function set inBufferSeek(isInBuffer:Boolean) {
            stub_setter("flash.net.NetStream", "inBufferSeek");
        }

        public function get info():NetStreamInfo {
            stub_getter("flash.net.NetStream", "info");
            return new NetStreamInfo();
        }


        public function get liveDelay(): Number {
            stub_getter("flash.net.NetStream", "liveDelay");
            return 0;

        };

        public function get maxPauseBufferTime(): Number {
            stub_getter("flash.net.NetStream", "maxPauseBufferTime");
            return 0;
        };

        public function set maxPauseBufferTime(time:Number) {
            stub_setter("flash.net.NetStream", "maxPauseBufferTime");
        };

        public function get multicastAvailabilitySendToAll():Boolean {
            stub_getter("flash.net.NetStream", "multicastAvailabilitySendToAll");
            return false;
        };

        public function set multicastAvailabilitySendToAll(toAll:Boolean) {
            stub_setter("flash.net.NetStream", "multicastAvailabilitySendToAll");
        };

        public function get multicastAvailabilityUpdatePeriod(): Number {
            stub_getter("flash.net.NetStream", "multicastAvailabilityUpdatePeriod");
            return 0;
        };

        public function set multicastAvailabilityUpdatePeriod(period:Number) {
            stub_setter("flash.net.NetStream", "multicastAvailabilityUpdatePeriod");
        };

        public function get multicastFetchPeriod():Number {
            stub_getter("flash.net.NetStream", "multicastFetchPeriod");
            return 0;
        };

        public function set multicastFetchPeriod(period:Number) {
            stub_setter("flash.net.NetStream", "multicastFetchPeriod");
        };

        public function get multicastInfo() : NetStreamMulticastInfo {
            stub_getter("flash.net.NetStream", "multicastInfo");
            return new NetStreamMulticastInfo();
        };

        public function get multicastPushNeighborLimit() : Number {
            stub_getter("flash.net.NetStream", "multicastPushNeighborLimit");
            return 0;
        };

        public function set multicastPushNeighborLimit(limit:Number) {
            stub_setter("flash.net.NetStream", "multicastPushNeighborLimit");
        };

        public function get multicastRelayMarginDuration() : Number {
            stub_getter("flash.net.NetStream", "multicastRelayMarginDuration");
            return 0;
        };

        public function set multicastRelayMarginDuration(dur: Number) {
            stub_setter("flash.net.NetStream", "multicastRelayMarginDuration");
        };

        public function get multicastWindowDuration() : Number {
            stub_getter("flash.net.NetStream", "multicastWindowDuration");
            return 0;

        };

        public function set multicastWindowDuration(dur:Number) {
            stub_setter("flash.net.NetStream", "multicastWindowDuration");
        };

        public function get nearNonce(): String {
            stub_getter("flash.net.NetStream", "nearNonce");
            return "";
        };

        public function get objectEncoding(): uint {
            stub_getter("flash.net.NetStream", "objectEncoding");
            return 0;
        };

        public function get peerStreams(): Array {
            stub_getter("flash.net.NetStream", "peerStreams");
            return [];
        };

        public function get soundTransform(): flash.media.SoundTransform {
            stub_getter("flash.net.NetStream", "soundTransform");
            return new flash.media.SoundTransform();
        };

        public function set soundTransform(tf:flash.media.SoundTransform) {
            stub_setter("flash.net.NetStream", "soundTransform");
        };

        public native function get time(): Number;

        public function get useHardwareDecoder(): Boolean {
            stub_getter("flash.net.NetStream", "useHardwareDecoder");
            return true;
        };

        public function set useHardwareDecoder(dec:Boolean) {
            stub_setter("flash.net.NetStream", "useHardwareDecoder");
        };

        [API("680")]
        public function get useJitterBuffer(): Boolean {
            stub_getter("flash.net.NetStream", "useJitterBuffer");
            return false;
        };

        [API("680")]
        public function set useJitterBuffer(jbuf:Boolean) {
            stub_setter("flash.net.NetStream", "useJitterBuffer");
        };

        public function get videoReliable(): Boolean {
            stub_getter("flash.net.NetStream", "videoReliable");
            return false;
        };

        public function set videoReliable(isReliable:Boolean) {
            stub_setter("flash.net.NetStream", "videoReliable");
        };

        public function get videoSampleAccess():Boolean {
            stub_getter("flash.net.NetStream", "videoSampleAccess");
            return false;
        };

        public function set videoSampleAccess(isReadable:Boolean) {
            stub_setter("flash.net.NetStream", "videoSampleAccess");
        };

        [API("674")]
        public function get videoStreamSettings(): VideoStreamSettings {
            stub_getter("flash.net.NetStream", "videoStreamSettings");
            return null;
        };

        [API("674")]
        public function set videoStreamSettings(settings: VideoStreamSettings) {
            stub_setter("flash.net.NetStream", "videoStreamSettings");
        };
    }
}
