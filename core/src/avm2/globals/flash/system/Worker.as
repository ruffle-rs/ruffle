package flash.system {

    import flash.events.Event;
    import flash.events.EventDispatcher;
    import flash.system.MessageChannel;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_method;

    [API("682")]
    [Ruffle(Abstract)]
    public final class Worker extends EventDispatcher {
        public static function get isSupported():Boolean {
            return false;
        }

        private static var _current:Worker;

        public static function get current():Worker {
            stub_getter("flash.system.Worker", "current");

            if (!_current) {
                _current = instantiateInternal();
            }

            return _current;
        }

        public native function createMessageChannel(receiver:Worker):MessageChannel;

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

        private static native function instantiateInternal():Worker;
    }
}
