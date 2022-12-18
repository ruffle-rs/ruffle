package flash.net {
    import flash.events.EventDispatcher;
    import __ruffle__.log_warn;

    // NOTE: this entire class is a stub.
    // Thankfully (hopefully) a lot of code like Mochicrypt doesn't actually require this to... well do anything.
    public class LocalConnection extends EventDispatcher {

        public var client: Object;

        public function LocalConnection() {
            this.client = this;
        }

        public function get domain():String {
            // FIXME - implement this - this is unrelated to the messaging functionality.
            return "localhost";
        }

        public function close(): void {}
        public function connect(connectionName:String): void {
            log_warn("LocalConnection.connect is not implemented");
        }

        public function send(connectionName: String, methodName: String, ... arguments): void {}

        public function allowDomain(... domains): void {}
        public function allowInsecureDomain(... domains): void {}
    }
}