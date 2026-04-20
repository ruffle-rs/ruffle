package flash.events {
    public class DataEvent extends TextEvent {
        public static const DATA:String = "data";
        public static const UPLOAD_COMPLETE_DATA:String = "uploadCompleteData";

        public function DataEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            data:String = ""
        ) {
            super(type, bubbles, cancelable, data);
        }

        // `DataEvent.data` seems to delegate to the superclass's (TextEvent's) `TextEvent.text`.
        public function get data():String {
            return super.text;
        }

        public function set data(value:String):void {
            super.text = value;
        }

        override public function clone():Event {
            return new DataEvent(this.type, this.bubbles, this.cancelable, this.data);
        }

        override public function toString():String {
            return this.formatToString(
                "DataEvent",
                "type",
                "bubbles",
                "cancelable",
                "eventPhase",
                "data"
            );
        }
    }
}
