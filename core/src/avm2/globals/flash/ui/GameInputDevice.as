package flash.ui {
    import flash.utils.ByteArray;

    [API("688")]
    [Ruffle(Abstract)]
    public final class GameInputDevice {
        public static const MAX_BUFFER_SIZE:int = 32000;

        private var _enabled:Boolean = false;
        private var _id:String = "";
        private var _name:String = "";
        private var _sampleInterval:int = 0;

        public function get enabled():Boolean {
            return this._enabled;
        }

        public function set enabled(val:Boolean):void {
            this._enabled = val;
        }

        public function get id():String {
            return this._id;
        }

        public function get name():String {
            return this._name;
        }

        public function get numControls():int {
            return 0;
        }

        public function get sampleInterval():int {
            return this._sampleInterval;
        }

        public function set sampleInterval(val:int):void {
            this._sampleInterval = val;
        }

        public function getControlAt(i:int):GameInputControl {
            if (i < 0 || i >= this.numControls) {
                Error.throwError(RangeError, 2006);
            }
            return null;
        }

        public function startCachingSamples(numSamples:int, controls:Vector.<String>):void {
        }

        public function stopCachingSamples():void {
        }

        public function getCachedSamples(data:ByteArray, append:Boolean = false):int {
            return 0;
        }
    }
}
