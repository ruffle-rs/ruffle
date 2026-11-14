package flash.sensors {
    import flash.events.EventDispatcher;

    [API("667")]
    public class Accelerometer extends EventDispatcher {
        private var _requestedUpdateInterval:Number = 0;

        public function Accelerometer() {
            super();
        }

        public static function get isSupported():Boolean {
            return false;
        }

        public function setRequestedUpdateInterval(interval:Number):void {
            if (interval <= 0 || isNaN(interval)) {
                Error.throwError(ArgumentError, 2004);
            }
            this._requestedUpdateInterval = interval;
        }

        public function get muted():Boolean {
            return true;
        }
    }
}
