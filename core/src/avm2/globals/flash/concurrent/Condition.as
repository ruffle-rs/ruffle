package flash.concurrent {
    [API("684")]
    public final class Condition {
        public static const isSupported:Boolean = false;

        private var _mutex:Mutex;

        public function Condition(mutex:Mutex) {
            if (mutex == null) {
                Error.throwError(TypeError, 2007, "mutex");
            }
            if (!isSupported) {
                throw new Error("Error #1520: Condition cannot be initialized.", 1520);
            }
            this._mutex = mutex;
        }

        public function get mutex():Mutex {
            return this._mutex;
        }

        public function notify():void {
        }

        public function notifyAll():void {
        }

        public function wait(timeout:Number = -1):Boolean {
            return false;
        }
    }
}
