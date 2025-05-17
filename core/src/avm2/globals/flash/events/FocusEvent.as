package flash.events {
    import flash.display.InteractiveObject;

    public class FocusEvent extends Event {
        public static const FOCUS_IN:String = "focusIn";
        public static const FOCUS_OUT:String = "focusOut";
        public static const KEY_FOCUS_CHANGE:String = "keyFocusChange";
        public static const MOUSE_FOCUS_CHANGE:String = "mouseFocusChange";

        private var _relatedObject:InteractiveObject;
        private var _shiftKey:Boolean;
        private var _keyCode:uint;
        private var _direction:String;
        private var _isRelatedObjectInaccessible:Boolean;

        public function FocusEvent(type:String, bubbles:Boolean = true, cancelable:Boolean = false, relatedObject:InteractiveObject = null,
            shiftKey:Boolean = false, keyCode:uint = 0, direction:String = "none") {
            super(type, bubbles, cancelable);
            this.relatedObject = relatedObject;
            this.shiftKey = shiftKey;
            this.keyCode = keyCode;
            this.direction = direction;
            this.isRelatedObjectInaccessible = false; // Unimplemented
        }

        public function get relatedObject():InteractiveObject {
            return this._relatedObject;
        }
        public function set relatedObject(value:InteractiveObject):void {
            this._relatedObject = value;
        }

        public function get shiftKey():Boolean {
            return this._shiftKey;
        }
        public function set shiftKey(value:Boolean):void {
            this._shiftKey = value;
        }

        public function get keyCode():uint {
            return this._keyCode;
        }
        public function set keyCode(value:uint):void {
            this._keyCode = value;
        }

        [API("661")]
        public function get direction():String {
            return this._direction;
        }
        [API("661")]
        public function set direction(value:String):void {
            this._direction = value;
        }

        [API("667")]
        public function get isRelatedObjectInaccessible():Boolean {
            return this._isRelatedObjectInaccessible;
        }
        [API("667")]
        public function set isRelatedObjectInaccessible(value:Boolean):void {
            this._isRelatedObjectInaccessible = value;
        }

        override public function clone():Event {
            return new FocusEvent(this.type, this.bubbles, this.cancelable, this.relatedObject, this.shiftKey, this.keyCode, this.direction);
        }

        override public function toString():String {
            return this.formatToString("FocusEvent", "type", "bubbles", "cancelable", "eventPhase", "relatedObject", "shiftKey", "keyCode");
        }
    }
}
