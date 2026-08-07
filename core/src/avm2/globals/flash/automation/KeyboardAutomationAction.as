package flash.automation {
    public class KeyboardAutomationAction extends AutomationAction {
        public static const KEY_DOWN:String = "keyDown";
        public static const KEY_UP:String = "keyUp";

        private var _keyCode:uint;

        public function KeyboardAutomationAction(type:String, keyCode:int = 0) {
            this.type = type
            this._keyCode = keyCode;
        }

        public function get keyCode():uint {
            return this._keyCode;
        }

        public function set keyCode(value:uint):void {
            this._keyCode = value;
        }
    }
}
