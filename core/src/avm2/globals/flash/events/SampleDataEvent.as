package flash.events {
    import flash.utils.ByteArray;

    public class SampleDataEvent extends Event {
        public static const SAMPLE_DATA:String = "sampleData";

        public var _position:Number;
        public var _data:ByteArray;

        public function SampleDataEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            theposition:Number = 0,
            thedata:ByteArray = null
        ) {
            super(type, bubbles, cancelable);
            this.position = theposition;
            this.data = thedata;
        }

        public function get position():Number {
            return this._position;
        }
        public function set position(value:Number):void {
            this._position = value;
        }

        public function get data():ByteArray {
            return this._data;
        }
        public function set data(value:ByteArray):void {
            this._data = value;
        }

        override public function clone():Event {
            return new SampleDataEvent(
                this.type,
                this.bubbles,
                this.cancelable,
                this.position,
                this.data
            );
        }

        override public function toString():String {
            return this.formatToString(
                "SampleDataEvent",
                "type",
                "bubbles",
                "cancelable",
                "eventPhase",
                "position",
                "data"
            );
        }
    }
}
