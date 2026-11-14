package flash.ui {
    import flash.events.EventDispatcher;
    import flash.utils.ByteArray;

    [API("688")]
    [Ruffle(Abstract)]
    public dynamic class GameInputControl extends EventDispatcher {
        private var _device:GameInputDevice = null;
        private var _enabled:Boolean = false;
        private var _id:String = "";
        private var _minValue:Number = 0;
        private var _maxValue:Number = 0;
        private var _value:Number = 0;

        public function get device():GameInputDevice {
            return this._device;
        }

        public function get enabled():Boolean {
            return this._enabled;
        }

        public function get id():String {
            return this._id;
        }

        public function get minValue():Number {
            return this._minValue;
        }

        public function get maxValue():Number {
            return this._maxValue;
        }

        public function get value():Number {
            return this._value;
        }

        public function getCachedSamples(data:ByteArray, append:Boolean = false):int {
            return 0;
        }
    }
}
