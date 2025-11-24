package flash.events {
    public class NetStatusEvent extends Event {
        public static const NET_STATUS:String = "netStatus";

        private var _info:Object;

        public function NetStatusEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            info:Object = null
        ) {
            super(type, bubbles, cancelable);
            this.info = info;
        }

        public function get info():Object {
            return this._info;
        }
        public function set info(value:Object):void {
            this._info = value;
        }

        override public function clone():Event {
            return new NetStatusEvent(this.type, this.bubbles, this.cancelable, this.info);
        }

        override public function toString():String {
            return this.formatToString(
                "NetStatusEvent",
                "type",
                "bubbles",
                "cancelable",
                "eventPhase",
                "info"
            );
        }
    }
}
