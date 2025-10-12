package flash.events {
    import flash.utils.ByteArray;

    public class SampleDataEvent extends Event {
        public static const SAMPLE_DATA:String = "sampleData"; // Defines the value of the type property of a SampleDataEvent event object.

        public var _position: Number; // The position of the data in the audio stream.

        [Ruffle(NativeAccessible)]
        public var _data: ByteArray; // The data in the audio stream.

        public function SampleDataEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, theposition:Number = 0, thedata:ByteArray = null) {
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

        // Creates a copy of the SampleDataEvent object and sets each property's value to match that of the original.
        override public function clone():Event {
            return new SampleDataEvent(this.type, this.bubbles, this.cancelable, this.position, this.data);
        }

        // Returns a string that contains all the properties of the SampleDataEvent object.
        override public function toString():String {
            return this.formatToString("SampleDataEvent", "type", "bubbles", "cancelable", "eventPhase", "position", "data");
        }
    }
}
