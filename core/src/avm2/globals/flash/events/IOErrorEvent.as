package flash.events {
    public class IOErrorEvent extends ErrorEvent {
        public static const IO_ERROR:String = "ioError";


        public function IOErrorEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, text:String = "", id:int = 0)
        {
            super(type,bubbles,cancelable,text,id);
        }

        override public function clone() : Event
        {
            return new IOErrorEvent(this.type,this.bubbles,this.cancelable,this.text,this.errorID);
        }

        override public function toString() : String
        {
            return this.formatToString("IOErrorEvent","type","bubbles","cancelable","eventPhase","text");
        }
    }
}
