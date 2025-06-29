package flash.system {

    import flash.utils.Dictionary;
    import flash.events.Event;
    import flash.events.EventDispatcher;
    import flash.system.MessageChannel;
    import __ruffle__.stub_method;

    [API("682")]
    public final class Worker extends EventDispatcher {
        public static const isSupported:Boolean = false;

        private static var _current:Worker;
        
        public static function get current():Worker {
            if (!_current) {
                _current = new Worker();
            }
            return _current;
        }

        public function createMessageChannel(received:Worker): MessageChannel {
            stub_method("flash.system.Worker", "createMessageChannel");

            return new MessageChannel();
        }

        public function setSharedProperty(key:String, value:*):void {
            stub_method("flash.system.Worker", "setSharedProperty");
        }

        public function getSharedProperty(key:String):* {
            stub_method("flash.system.Worker", "getSharedProperty");
        }

        public function start():void {
            this.dispatchEvent(new Event(Event.WORKER_STATE));

            stub_method("flash.system.Worker", "start");
        }
    }
}
