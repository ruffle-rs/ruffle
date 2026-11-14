package flash.system {

    import flash.events.EventDispatcher;

    [API("682")]
    [Ruffle(Abstract)]
    public final class MessageChannel extends EventDispatcher {
        public function send(arg:*, queueLimit:int = -1):void {
        }

        public function get state():String {
            return "open";
        }
    }
}
