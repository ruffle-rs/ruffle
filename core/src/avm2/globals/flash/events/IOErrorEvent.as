package flash.events {
    public class IOErrorEvent extends ErrorEvent {
        public static const IO_ERROR:String = "ioError";

        // These next three are undocumented.
        public static const NETWORK_ERROR:String = "networkError";
        public static const DISK_ERROR:String = "diskError";
        public static const VERIFY_ERROR:String = "verifyError";

        [API("668")]
        public static const STANDARD_ERROR_IO_ERROR:String = "standardErrorIoError";

        [API("668")]
        public static const STANDARD_INPUT_IO_ERROR:String = "standardInputIoError";

        [API("668")]
        public static const STANDARD_OUTPUT_IO_ERROR:String = "standardOutputIoError";

        public function IOErrorEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            text:String = "",
            id:int = 0
        ) {
            super(type, bubbles, cancelable, text, id);
        }

        override public function clone():Event {
            return new IOErrorEvent(
                this.type,
                this.bubbles,
                this.cancelable,
                this.text,
                this.errorID
            );
        }

        override public function toString():String {
            return this.formatToString(
                "IOErrorEvent",
                "type",
                "bubbles",
                "cancelable",
                "eventPhase",
                "text"
            );
        }
    }
}
