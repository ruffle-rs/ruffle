package flash.events {
    [API("674")]
    public class OutputProgressEvent extends Event {
        public static const OUTPUT_PROGRESS:String = "outputProgress";

        private var _bytesPending:Number;
        private var _bytesTotal:Number;

        public function OutputProgressEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, bytesPending:Number = 0.0, bytesTotal:Number = 0.0) {
            super(type, bubbles, cancelable);

            this._bytesPending = bytesPending;
            this._bytesTotal = bytesTotal;
        }

        public function get bytesPending():Number {
            return this._bytesPending;
        }
        public function set bytesPending(value:Number):void {
            this._bytesPending = value;
        }

        public function get bytesTotal():Number {
            return this._bytesTotal;
        }
        public function set bytesTotal(value:Number):void {
            this._bytesTotal = value;
        }

        override public function clone():Event {
            return new OutputProgressEvent(type, bubbles, cancelable, this._bytesPending, this._bytesTotal);
        }

        override public function toString():String {
            return formatToString("OutputProgressEvent", "type", "bubbles", "cancelable", "eventPhase", "bytesPending", "bytesTotal");
        }
    }
}
