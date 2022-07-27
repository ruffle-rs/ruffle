package flash.events {
    public class ErrorEvent extends TextEvent {

        public static const ERROR:String = "error";

        private var _errorID:int;

        public function ErrorEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, text:String = "", id:int = 0)
        {
            super(type,bubbles,cancelable,text);
            this._errorID = id;
        }

        public function get errorID() : int
        {
            return this._errorID;
        }

        override public function clone() : Event
        {
            return new ErrorEvent(this.type,this.bubbles,this.cancelable,this.text,this._errorID);
        }

        override public function toString() : String
        {
            return this.formatToString("ErrorEvent","type","bubbles","cancelable","eventPhase","text");
        }
    }
}
