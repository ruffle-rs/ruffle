package flash.display {
    import __ruffle__.stub_setter;

    import flash.accessibility.AccessibilityImplementation;
    import flash.geom.Rectangle;
    import flash.ui.ContextMenu;

    [Ruffle(SuperInitializer)]
    public class InteractiveObject extends DisplayObject {
        private var _accessibilityImpl:AccessibilityImplementation = null;
        private var _needsSoftKeyboard:Boolean = false;
        private var _softKeyboardInputAreaOfInterest:Rectangle = new Rectangle();

        public function InteractiveObject() {
            throw new Error("You cannot directly construct InteractiveObject.")
        }

        public function get accessibilityImplementation():AccessibilityImplementation {
            return this._accessibilityImpl;
        }
        public function set accessibilityImplementation(value:AccessibilityImplementation):void {
            stub_setter("flash.display.InteractiveObject", "accessibilityImplementation");
            this._accessibilityImpl = value;
        }

        public native function get mouseEnabled():Boolean;
        public native function set mouseEnabled(value:Boolean):void;

        [API("670")]
        public function get needsSoftKeyboard():Boolean {
            return this._needsSoftKeyboard;
        }
        [API("670")]
        public function set needsSoftKeyboard(value:Boolean):void {
            stub_setter("flash.display.InteractiveObject", "needsSoftKeyboard");
            this._needsSoftKeyboard = value;
        }

        [API("670")]
        public function get softKeyboardInputAreaOfInterest():Rectangle {
            return this._softKeyboardInputAreaOfInterest;
        }
        [API("670")]
        public function set softKeyboardInputAreaOfInterest(value:Rectangle):void {
            stub_setter("flash.display.InteractiveObject", "softKeyboardInputAreaOfInterest");
            this._softKeyboardInputAreaOfInterest = value;
        }

        public native function get doubleClickEnabled():Boolean;
        public native function set doubleClickEnabled(value:Boolean):void;

        public native function get contextMenu():ContextMenu;
        public native function set contextMenu(cm:ContextMenu):void;

        public native function get tabEnabled():Boolean;
        public native function set tabEnabled(value:Boolean):void;

        public native function get tabIndex():int;
        public native function set tabIndex(index:int):void;

        public native function get focusRect():Object;
        public native function set focusRect(value:Object):void;
    }
}
