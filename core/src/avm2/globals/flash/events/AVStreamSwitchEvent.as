package flash.events {
    [API("688")]
    public class AVStreamSwitchEvent extends Event {
        public static const AV_STREAM_SWITCH:String = "avStreamSwitch";
        public static const ABR_SWITCH:int = 0;
        public static const PERIOD_SWITCH:int = 1;

        private var _time:Number;
        private var _switchType:int;
        private var _bitrate:int;
        private var _description:String;
        private var _userData:int;

        public function AVStreamSwitchEvent(type:String = "avStreamSwitch", bubbles:Boolean = false, cancelable:Boolean = false, time:Number = 0.0, switchType:int = 0, bitrate:int = 0, description:String = "", userData:int = 0) {
            super(type, bubbles, cancelable);
            this._time = time;
            this._switchType = switchType;
            this._bitrate = bitrate;
            this._description = description;
            this._userData = userData;
        }

        public function get time():Number {
            return this._time;
        }

        public function get switchType():int {
            return this._switchType;
        }

        public function get bitrate():int {
            return this._bitrate;
        }

        public function get description():String {
            return this._description;
        }

        public function get userData():int {
            return this._userData;
        }
    }
}
