package flash.system {
    import flash.events.EventDispatcher;

    [API("682")]
    public final class MessageChannel extends EventDispatcher {
        public function MessageChannel() {
            super();
        }

        public function send(arg:*, queueLimit:int = -1):void {
            stub_method("flash.system.MessageChannel", "send");
        }
    }
}
