package flash.net {
    import flash.events.EventDispatcher;
    import flash.errors.IOError;
    import __ruffle__.stub_method;

    [Ruffle(InstanceAllocator)]
    public class NetConnection extends EventDispatcher {

        public static var defaultObjectEncoding:uint = 3;

        public var objectEncoding:uint = NetConnection.defaultObjectEncoding;
        public var client:Object = this;
        public var maxPeerConnections:uint = 8;
        public var proxyType:String = "none";


        public native function connect(command:String, ... arguments):void;

        public native function addHeader(operation:String, mustUnderstand:Boolean = false, param:Object = null):void;

        public native function call(command:String, responder:Responder, ... arguments):void;
        
        public native function close():void;

        public native function get connected():Boolean;
        public native function get connectedProxyType():String;
        public native function get farID():String;
        public native function get farNonce():String;
        public native function get nearID():String;
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
    }
}
