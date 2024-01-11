package flash.events
{
    public class KeyboardEvent extends Event
    {
        public static const KEY_DOWN:String = "keyDown";
        public static const KEY_UP:String = "keyUp";
        private var _charCode:uint;
        private var _keyCode:uint;
        private var _keyLocation:uint;
        private var _ctrlKey:Boolean;
        private var _altKey:Boolean;
        private var _shiftKey:Boolean;
        private var _controlKey:Boolean;
        private var _commandKey:Boolean;

        public function KeyboardEvent(type:String, 
                                      bubbles:Boolean = true, 
                                      cancelable:Boolean = false, 
                                      charCodeValue:uint = 0, 
                                      keyCodeValue:uint = 0, 
                                      keyLocationValue:uint = 0, 
                                      ctrlKeyValue:Boolean = false, 
                                      altKeyValue:Boolean = false, 
                                      shiftKeyValue:Boolean = false, 
                                      controlKeyValue:Boolean = false, 
                                      commandKeyValue:Boolean = false)
        {
            super(type,bubbles,cancelable);
            this._charCode = charCodeValue;
            this._keyCode = keyCodeValue;
            this._keyLocation = keyLocationValue;
            this._ctrlKey = ctrlKeyValue;
            this._altKey = altKeyValue;
            this._shiftKey = shiftKeyValue;
            this._controlKey = controlKeyValue;
            this._commandKey = commandKeyValue;
        }

        public function get charCode():uint {
            return this._charCode;
        }
        public function set charCode(val:uint) {
            this._charCode = val;
        }

        public function get keyCode():uint {
            return this._keyCode;
        }
        public function set keyCode(val:uint) {
            this._keyCode = val;
        }

        public function get keyLocation():uint {
            return this._keyLocation;
        }
        public function set keyLocation(val:uint):void {
            this._keyLocation = val;
        }

        public function get ctrlKey():Boolean {
            return this._ctrlKey;
        }
        public function set ctrlKey(val:Boolean) {
            this._ctrlKey = val;
        }

        public function get altKey():Boolean {
            return this._altKey;
        }
        public function set altKey(val:Boolean) {
            this._altKey = val;
        }

        public function get shiftKey():Boolean {
            return this._shiftKey;
        }
        public function set shiftKey(val:Boolean) {
            this._shiftKey = val;
        }

        public function get controlKey():Boolean {
            return this._controlKey;
        }
        public function set controlKey(val:Boolean) {
            this._controlKey = val;
        }

        public function get commandKey():Boolean {
            return this._commandKey;
        }
        public function set commandKey(val:Boolean) {
            this._commandKey = val;
        }

        override public function clone() : Event
        {
            return new KeyboardEvent(this.type,this.bubbles,this.cancelable,this._charCode,this._keyCode,this._keyLocation,this._ctrlKey,this._altKey,this._shiftKey,this._controlKey,this._commandKey);
        }

        override public function toString(): String {
            return this.formatToString("KeyboardEvent", "type", "bubbles", "cancelable", "eventPhase", "charCode", "keyCode", "keyLocation", "ctrlKey", "altKey", "shiftKey");
        }

        public native function updateAfterEvent():void;
    }
}
