package flash.events {
    public class NetDataEvent extends Event {
        public static const MEDIA_TYPE_DATA:String = "mediaTypeData";

        private var _timestamp:Number;
        private var _info:Object;

        public function NetDataEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            timestamp:Number = 0,
            info:Object = null
        ) {
            super(type, bubbles, cancelable);
            this._timestamp = timestamp;
            this._info = info;
        }

        override public function clone():Event {
            return new NetDataEvent(
                this.type,
                this.bubbles,
                this.cancelable,
                this.timestamp,
                this.info
            );
        }

        override public function toString():String {
            return this.formatToString(
                "NetDataEvent",
                "type",
                "bubbles",
                "cancelable",
                "eventPhase",
                "timestamp"
            );
        }

        public function get timestamp():Number {
            return this._timestamp;
        }

        public function get info():Object {
            return this._info;
        }
    }
}
