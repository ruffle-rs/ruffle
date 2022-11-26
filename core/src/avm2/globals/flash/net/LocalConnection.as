package flash.net {
    import flash.events.EventDispatcher;
    public class LocalConnection extends EventDispatcher {
        public function get domain():String {
            // FIXME - implement this
            return "localhost";
        }
    }
}