package flash.net {
    import flash.events.EventDispatcher;
    import flash.errors.IOError;
    import __ruffle__.stub_method;

    public class NetConnection extends EventDispatcher {

        public static var defaultObjectEncoding:uint = 3;

        public function NetConnection() {}
        public var objectEncoding:uint = NetConnection.defaultObjectEncoding; // NOPMD WronglyNamedVariable

        public function connect(command:String, ... arguments):void {
            stub_method("flash.net.NetConnection", "connect");
        }

        public function call(command:String, responder:Responder, ... arguments):void {
            stub_method("flash.net.NetConnection", "call");
        }
    }
}