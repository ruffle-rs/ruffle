package flash.events {
    [API("661")]
    public class NativeWindowDisplayStateEvent extends Event {
        public static const DISPLAY_STATE_CHANGING:String = "displayStateChanging";
        public static const DISPLAY_STATE_CHANGE:String = "displayStateChange";

        private var _beforeDisplayState:String;
        private var _afterDisplayState:String;

        public function NativeWindowDisplayStateEvent(
            type:String,
            bubbles:Boolean = true,
            cancelable:Boolean = false,
            beforeDisplayState:String = "",
            afterDisplayState:String = ""
        ) {
            super(type, bubbles, cancelable);
            this._beforeDisplayState = beforeDisplayState;
            this._afterDisplayState = afterDisplayState;
        }

        override public function clone():Event {
            return new NativeWindowDisplayStateEvent(
                this.type,
                this.bubbles,
                this.cancelable,
                this.beforeDisplayState,
                this.afterDisplayState
            );
        }

        override public function toString():String {
            return this.formatToString(
                "NativeWindowDisplayStateEvent",
                "type",
                "bubbles",
                "cancelable",
                "beforeDisplayState",
                "afterDisplayState"
            );
        }

        public function get beforeDisplayState():String {
            return this._beforeDisplayState;
        }

        public function get afterDisplayState():String {
            return this._afterDisplayState;
        }
    }
}
