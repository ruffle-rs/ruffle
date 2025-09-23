package flash.system {
    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;

    import flash.events.EventDispatcher;

    [API("682")]
    [Ruffle(Abstract)]
    public final class MessageChannel extends EventDispatcher {
        public function send(arg:*, queueLimit:int = -1):void {
            stub_method("flash.system.MessageChannel", "send");
        }

        public function get state():String {
            stub_getter("flash.system.MessageChannel", "state");

            return "open";
        }
    }
}
