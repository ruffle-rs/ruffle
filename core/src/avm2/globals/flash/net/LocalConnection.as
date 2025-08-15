package flash.net {
    import flash.events.EventDispatcher;
    import flash.events.StatusEvent;
    import flash.utils.setTimeout;
    import __ruffle__.stub_method;

    [Ruffle(InstanceAllocator)]
    public class LocalConnection extends EventDispatcher {
        public function LocalConnection() {
            super();
        }

        [API("667")]
        public static function get isSupported():Boolean {
            return true;
        }

        public native function get domain():String;

        public native function close():void;

        public native function connect(connectionName:String):void;

        public native function send(connectionName: String, methodName: String, ... arguments):void;

        public native function get client():Object;
        public native function set client(client:Object):void;

        public function allowDomain(... domains): void {
            stub_method("flash.net.LocalConnection", "allowDomain");
        }

        public function allowInsecureDomain(... domains): void {
            stub_method("flash.net.LocalConnection", "allowInsecureDomain");
        }
    }
}
