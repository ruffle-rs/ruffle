package flash.net {
    import flash.events.EventDispatcher;
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;

    // NOTE: this entire class is a stub.
    // Thankfully (hopefully) a lot of code like Mochicrypt doesn't actually require this to... well do anything.
    public class LocalConnection extends EventDispatcher {

        public var client: Object;

        public function LocalConnection() {
            this.client = this;
        }

        public native function get domain():String;

        public function close(): void {
            stub_method("flash.net.LocalConnection", "close");
        }

        public function connect(connectionName:String): void {
            stub_method("flash.net.LocalConnection", "connect");
        }

        public native function send(connectionName: String, methodName: String, ... arguments);

        public function allowDomain(... domains): void {
            stub_method("flash.net.LocalConnection", "allowDomain");
        }

        public function allowInsecureDomain(... domains): void {
            stub_method("flash.net.LocalConnection", "allowInsecureDomain");
        }
    }
}
