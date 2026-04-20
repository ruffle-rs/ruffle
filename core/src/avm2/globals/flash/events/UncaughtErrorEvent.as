package flash.events {
    public class UncaughtErrorEvent extends ErrorEvent {
        public static const UNCAUGHT_ERROR:String = "uncaughtError";

        private var _error:*;

        public function UncaughtErrorEvent(
            type:String = "uncaughtError",
            bubbles:Boolean = true,
            cancelable:Boolean = true,
            error_in:* = null
        ) {
            super(type, bubbles, cancelable);
            this._error = error_in;
        }

        override public function clone():Event {
            return new UncaughtErrorEvent(this.type, this.bubbles, this.cancelable, this.error);
        }

        override public function toString():String {
            return this.formatToString(
                "UncaughtErrorEvent",
                "type",
                "bubbles",
                "cancelable",
                "eventPhase",
                "error"
            );
        }

        public function get error():* {
            return this._error;
        }
    }
}
