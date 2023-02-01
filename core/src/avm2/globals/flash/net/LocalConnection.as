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

        public function get domain():String {
            // FIXME - implement this - this is unrelated to the messaging functionality.
            stub_getter("flash.net.LocalConnection", "domain");
            return "localhost";
        }

        public function close(): void {
            stub_method("flash.net.LocalConnection", "close");
        }

        public function connect(connectionName:String): void {
            stub_method("flash.net.LocalConnection", "connect");
        }

        public function send(connectionName: String, methodName: String, ... arguments): void {
            stub_method("flash.net.LocalConnection", "send");
        }

        public function allowDomain(... domains): void {
            stub_method("flash.net.LocalConnection", "allowDomain");
        }

        public function allowInsecureDomain(... domains): void {
            stub_method("flash.net.LocalConnection", "allowInsecureDomain");
        }
    }
}