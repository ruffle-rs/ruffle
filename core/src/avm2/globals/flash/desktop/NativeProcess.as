package flash.desktop {
    import flash.events.EventDispatcher;

    [API("668")]
    public class NativeProcess extends EventDispatcher {
        private var _running:Boolean = false;

        public function NativeProcess() {
            super();
        }

        public static function get isSupported():Boolean {
            return false;
        }

        public function get running():Boolean {
            return this._running;
        }

        public function start(info:NativeProcessStartupInfo):void {
            if (info == null) {
                Error.throwError(TypeError, 2007, "info");
            }
        }

        public function exit(force:Boolean = false):void {
            this._running = false;
        }

        public function closeInput():void {
        }
    }
}
