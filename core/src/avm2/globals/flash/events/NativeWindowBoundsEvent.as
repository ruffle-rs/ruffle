package flash.events {
    import flash.geom.Rectangle;

    [API("661")]
    public class NativeWindowBoundsEvent extends Event {
        public static const MOVING:String = "moving";
        public static const MOVE:String = "move";
        public static const RESIZING:String = "resizing";
        public static const RESIZE:String = "resize";

        private var _beforeBounds:Rectangle;
        private var _afterBounds:Rectangle;

        public function NativeWindowBoundsEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            beforeBounds:Rectangle = null,
            afterBounds:Rectangle = null
        ) {
            super(type, bubbles, cancelable);
            this._beforeBounds = beforeBounds;
            this._afterBounds = afterBounds;
        }

        override public function clone():Event {
            return new NativeWindowBoundsEvent(this.type, this.bubbles, this.cancelable, this.beforeBounds, this.afterBounds);
        }

        override public function toString():String {
            // According to the documentation, the format should be:
            // [NativeWindowBoundsEvent type=value bubbles=value cancelable=value previousDisplayState=value currentDisplayState=value]
            // but it seems that previousDisplayState and currentDisplayState doesn't exist.
            // It's likely a mistake in the documentation.
            return this.formatToString("NativeWindowBoundsEvent", "type", "bubbles", "cancelable", "beforeBounds", "afterBounds");
        }

        public function get beforeBounds():Rectangle {
            return this._beforeBounds;
        }

        public function get afterBounds():Rectangle {
            return this._afterBounds;
        }
    }
}
