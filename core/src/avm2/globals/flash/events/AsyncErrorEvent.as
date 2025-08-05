package flash.events {
    public class AsyncErrorEvent extends ErrorEvent {
        public static const ASYNC_ERROR:String = "asyncError";

        public var error:Error;

        public function AsyncErrorEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, text:String = "", error:Error = null) {
            super(type, bubbles, cancelable, text);
            this.error = error;
        }

        override public function clone():Event {
            return new AsyncErrorEvent(this.type, this.bubbles, this.cancelable, this.text, this.error);
        }

        override public function toString():String {
            return this.formatToString("AsyncErrorEvent", "type", "bubbles", "cancelable", "eventPhase", "text", "error");
        }
    }
}
