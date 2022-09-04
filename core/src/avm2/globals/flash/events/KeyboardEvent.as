package flash.events
{
    public class KeyboardEvent extends Event
    {
        public static const KEY_DOWN:String = "keyDown";
        public static const KEY_UP:String = "keyUp";
        private var _charCode:uint;
        private var _keyCode:uint;

        public function KeyboardEvent(type:String, 
                                      bubbles:Boolean = true, 
                                      cancelable:Boolean = false, 
                                      charCodeValue:uint = 0, 
                                      keyCodeValue:uint = 0, 
                                      keyLocationValue:uint = 0, 
                                      ctrlKeyValue:Boolean = false, 
                                      altKeyValue:Boolean = false, 
                                      shiftKeyValue:Boolean = false)
        {
            super(type,bubbles,cancelable);
            this._charCode = charCodeValue;
            this._keyCode = keyCodeValue;
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

        override public function clone() : Event
        {
            return new KeyboardEvent(this.type,this.bubbles,this.cancelable);
        }
    }
}
