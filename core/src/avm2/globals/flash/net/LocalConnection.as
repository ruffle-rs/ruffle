package flash.net {
    import flash.events.EventDispatcher;
    import flash.events.StatusEvent;
    import flash.utils.setTimeout;
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;

    [Ruffle(InstanceAllocator)]
    public class LocalConnection extends EventDispatcher {

        public var client: Object;

        public function LocalConnection() {
            this.client = this;
        }

        [API("667")]
        public static function get isSupported():Boolean {
            return true;
        }

        public native function get domain():String;

        public native function close():void;

        public native function connect(connectionName:String):void;

        public function send(connectionName: String, methodName: String, ... arguments):void {
            if (connectionName === null) {
                throw new TypeError("Error #2007: Parameter connectionName must be non-null.", 2007);
            }
            if (methodName === null) {
                throw new TypeError("Error #2007: Parameter methodName must be non-null.", 2007);
            }

            var self = this;
            setTimeout(function() {
                self.send_internal(connectionName, methodName, arguments);
            }, 0);
        }

        private native function send_internal(connectionName: String, methodName: String, args: Array):void;

        public function allowDomain(... domains): void {
            stub_method("flash.net.LocalConnection", "allowDomain");
        }

        public function allowInsecureDomain(... domains): void {
            stub_method("flash.net.LocalConnection", "allowInsecureDomain");
        }
    }
}
