package flash.net {
    import flash.events.EventDispatcher;
    import flash.errors.IOError;
    import __ruffle__.stub_method;

    public class NetConnection extends EventDispatcher {

        public static var defaultObjectEncoding:uint = 3;

        public var objectEncoding:uint = NetConnection.defaultObjectEncoding;


        public native function connect(command:String, ... arguments):void;

        public function addHeader(operation:String, mustUnderstand:Boolean = false, param:Object = null):void {
            stub_method("flash.net.NetConnection", "addHeader");
        }

        public function call(command:String, responder:Responder, ... arguments):void {
            stub_method("flash.net.NetConnection", "call");
        }
        
        public function close():void {
            stub_method("flash.net.NetConnection", "close");
        }
    }
}
