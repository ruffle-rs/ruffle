package flash.events{
    import flash.display.BitmapData;
    import flash.utils.ByteArray;

    [API("662")]
    public class ShaderEvent extends Event {
        public static const COMPLETE:String = "complete";

        private var _vector:Vector.<Number>;
        private var _bitmapData:BitmapData;
        private var _byteArray:ByteArray;

        public function ShaderEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            bitmap:BitmapData = null,
            array:ByteArray = null,
            vector:Vector.<Number> = null
        ) {
            super(type, bubbles, cancelable);
            this._bitmapData = bitmap;
            this._byteArray = array;
            this._vector = vector;
        }

        public function get bitmapData():BitmapData {
            return this._bitmapData;
        }
        public function set bitmapData(bitmapData:BitmapData) {
            this._bitmapData = bitmapData;
        }

        public function get byteArray():ByteArray {
            return this._byteArray;
        }
        public function set byteArray(byteArray:ByteArray) {
            this._byteArray = byteArray;
        }

        public function get vector():Vector.<Number> {
            return this._vector;
        }
        public function set vector(vector:Vector.<Number>) {
            this._vector = vector;
        }

        override public function clone():Event {
            return new ShaderEvent(
                this.type,
                this.bubbles,
                this.cancelable,
                this.bitmapData,
                this.byteArray,
                this.vector
            );
        }

        override public function toString():String {
            return this.formatToString(
                "ShaderEvent",
                "type",
                "bubbles",
                "cancelable",
                "eventPhase",
                "bitmapData",
                "byteArray",
                "vector"
            );
        }
    }
}
