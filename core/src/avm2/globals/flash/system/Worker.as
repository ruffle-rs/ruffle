package flash.system {
    import flash.events.Event;
    import flash.events.EventDispatcher;
    import flash.system.MessageChannel;

    [API("682")]
    [Ruffle(Abstract)]
    public final class Worker extends EventDispatcher {
        public static function get isSupported():Boolean {
            return false;
        }

        private static var _current:Worker;
        private var _sharedProperties:Object = new Object();
        private var _state:String = WorkerState.RUNNING;

        public static function get current():Worker {
            if (!_current) {
                _current = instantiateInternal();
            }

            return _current;
        }

        public function get isPrimordial():Boolean {
            return this == current;
        }

        public function get state():String {
            return this._state;
        }

        public native function createMessageChannel(receiver:Worker):MessageChannel;

        public function setSharedProperty(key:String, value:*):void {
            if (key == null) {
                Error.throwError(TypeError, 2007, "key");
            }
            this._sharedProperties[key] = value;
        }

        public function getSharedProperty(key:String):* {
            if (key == null) {
                Error.throwError(TypeError, 2007, "key");
            }
            return this._sharedProperties[key];
        }

        public function start():void {
            this._state = WorkerState.RUNNING;
            this.dispatchEvent(new Event(Event.WORKER_STATE));
        }

        public function terminate():Boolean {
            this._state = WorkerState.TERMINATED;
            this.dispatchEvent(new Event(Event.WORKER_STATE));
            return false;
        }

        private static native function instantiateInternal():Worker;
    }
}
