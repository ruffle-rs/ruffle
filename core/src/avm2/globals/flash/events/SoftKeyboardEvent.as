package flash.events {
    import flash.display.InteractiveObject;

    public class SoftKeyboardEvent extends Event {
        public static const SOFT_KEYBOARD_ACTIVATE:String = "softKeyboardActivate";
        public static const SOFT_KEYBOARD_ACTIVATING:String = "softKeyboardActivating";
        public static const SOFT_KEYBOARD_DEACTIVATE:String = "softKeyboardDeactivate";

        private var _relatedObject:InteractiveObject;

        public function SoftKeyboardEvent(type:String, bubbles:Boolean, cancelable:Boolean, relatedObjectVal:InteractiveObject, triggerTypeVal:String) {
            super(type, bubbles, cancelable);
            this.relatedObject = relatedObjectVal;
            this._triggerType = triggerTypeVal;
        }

        override public function clone():Event {
            return new SoftKeyboardEvent(this.type, this.bubbles, this.cancelable, this.relatedObject, this.triggerType);
        }

        override public function toString():String {
            return this.formatToString("SoftKeyboardEvent", "type", "bubbles", "cancelable", "eventPhase", "relatedObject", "triggerType");
        }

        public function get relatedObject():InteractiveObject {
            return this._relatedObject;
        }
        public function set relatedObject(value:InteractiveObject):void {
            this._relatedObject = value;
        }

        public function get triggerType():String {
            return this._triggerType;
        }
    }
}
