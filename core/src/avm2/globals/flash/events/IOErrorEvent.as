package flash.events {
    public class IOErrorEvent extends ErrorEvent {
        // Defines the value of the `type` property of an `ioError` event object.
        public static const IO_ERROR:String = "ioError";

        // These next three are undocumented.
        public static const NETWORK_ERROR:String = "networkError";
        public static const DISK_ERROR:String = "diskError";
        public static const VERIFY_ERROR:String = "verifyError";

        // The standardErrorIoError event is dispatched when an error occurs while
        // reading data from the standardError stream of a NativeProcess object.
        [API("668")]
        public static const STANDARD_ERROR_IO_ERROR:String = "standardErrorIoError";

        // The standardInputIoError event is dispatched when an error occurs while
        // writing data to the standardInput of a NativeProcess object.
        [API("668")]
        public static const STANDARD_INPUT_IO_ERROR:String = "standardInputIoError";

        //The standardOutputIoError event is dispatched when an error occurs while
        // reading data from the standardOutput stream of a NativeProcess object.
        [API("668")]
        public static const STANDARD_OUTPUT_IO_ERROR:String = "standardOutputIoError";


        public function IOErrorEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, text:String = "", id:int = 0)
        {
            super(type,bubbles,cancelable,text,id);
        }

        override public function clone() : Event
        {
            return new IOErrorEvent(this.type,this.bubbles,this.cancelable,this.text,this.errorID);
        }

        override public function toString() : String
        {
            return this.formatToString("IOErrorEvent","type","bubbles","cancelable","eventPhase","text");
        }
    }
}
