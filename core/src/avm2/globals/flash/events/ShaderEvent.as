package flash.events{
    import flash.display.BitmapData;
    import flash.utils.ByteArray;

    public class ShaderEvent extends Event {
        public static const COMPLETE:String = "complete";

        public var vector:Vector.<Number>;
        public var bitmapData:BitmapData;
        public var byteArray:ByteArray;

        public function ShaderEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            bitmap:BitmapData = null,
            array:ByteArray = null,
            vector:Vector.<Number> = null
        ) {
            super(type, bubbles, cancelable);
            this.bitmapData = bitmap;
            this.byteArray = array;
            this.vector = vector;
        }

        override public function clone():Event {
            return new ShaderEvent(
                this.type,
                this.bubbles,
                this.cancelable,
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
