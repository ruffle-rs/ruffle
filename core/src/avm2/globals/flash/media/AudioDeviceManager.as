package flash.media {
    import flash.events.EventDispatcher;

    [API("725")]
    public final class AudioDeviceManager extends EventDispatcher {
        private static var _instance:AudioDeviceManager;
        private var _selectedDeviceIndex:int = 0;

        public function AudioDeviceManager() {
            super();
        }

        public static function get isSupported():Boolean {
            return true;
        }

        public static function get audioDeviceManager():AudioDeviceManager {
            if (_instance == null) {
                _instance = new AudioDeviceManager();
            }
            return _instance;
        }

        public function get deviceNames():Array {
            return ["Default Audio Output"];
        }

        public function get selectedDeviceIndex():int {
            return this._selectedDeviceIndex;
        }

        public function set selectedDeviceIndex(value:int):void {
            if (value < 0 || value >= this.deviceNames.length) {
                Error.throwError(RangeError, 2006);
            }
            this._selectedDeviceIndex = value;
        }
    }
}
