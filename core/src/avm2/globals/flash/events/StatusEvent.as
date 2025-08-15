package flash.events {
    public class StatusEvent extends Event {
        public static const STATUS:String = "status";

        private var _code:String;
        private var _level:String;

        public function StatusEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, code:String = "", level:String = "") {
            super(type, bubbles, cancelable);
            this.code = code;
            this.level = level;
        }

        public function get code():String {
            return this._code;
        }
        public function set code(value:String):void {
            this._code = value;
        }

        public function get level():String {
            return this._level;
        }
        public function set level(value:String):void {
            this._level = value;
        }

        override public function clone():Event {
            return new StatusEvent(this.type, this.bubbles, this.cancelable, this.code, this.level);
        }

        override public function toString():String {
            return this.formatToString("StatusEvent", "type", "bubbles", "cancelable", "eventPhase", "code", "level");
        }
    }
}
