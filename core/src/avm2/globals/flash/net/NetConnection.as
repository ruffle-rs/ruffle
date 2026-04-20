package flash.net {
    import flash.events.EventDispatcher;
    import flash.errors.IOError;

    [Ruffle(InstanceAllocator)]
    public class NetConnection extends EventDispatcher {
        public static var defaultObjectEncoding:uint = 3;

        private var _objectEncoding:uint = NetConnection.defaultObjectEncoding;
        private var _client:Object = this;
        private var _maxPeerConnections:uint = 8;
        private var _proxyType:String = "none";

        public native function connect(command:String, ...arguments):void;

        public native function addHeader(operation:String, mustUnderstand:Boolean = false, param:Object = null):void;

        public native function call(command:String, responder:Responder, ...arguments):void;

        public native function close():void;

        public native function get connected():Boolean;
        public native function get connectedProxyType():String;

        [API("667")]
        public native function get farID():String;
        [API("667")]
        public native function get farNonce():String;
        [API("667")]
        public native function get nearID():String;
        [API("667")]
        public native function get nearNonce():String;

        public native function get protocol():String;
        public native function get uri():String;
        public native function get usingTLS():Boolean;

        public function get unconnectedPeerStreams():Array {
            if (this.connected) {
                // [NA] Arguably this isn't a stub as it can't ever be anything else in our current implementation...
                return [];
            } else {
                throw new ArgumentError("Error #2126: NetConnection object must be connected.", 2126);
            }
        }

        public function get objectEncoding():uint {
            return this._objectEncoding;
        }
        public function set objectEncoding(value:uint):void {
            // TODO do validation
            this._objectEncoding = value;
        }

        public function get client():Object {
            return this._client;
        }
        public function set client(value:Object):void {
            // TODO do validation
            this._client = value;
        }

        [API("667")]
        public function get maxPeerConnections():uint {
            return this._maxPeerConnections;
        }

        [API("667")]
        public function set maxPeerConnections(value:uint):void {
            this._maxPeerConnections = value;
        }

        public function get proxyType():String {
            return this._proxyType;
        }
        public function set proxyType(value:String):void {
            // TODO do validation
            this._proxyType = value;
        }
    }
}
