package flash.events {
    public class ProgressEvent extends Event {
        public static const PROGRESS:String = "progress";
        public static const SOCKET_DATA:String = "socketData";

        public var bytesLoaded:Number;
        public var bytesTotal:Number;

        public function ProgressEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, bytesLoaded:Number = 0, bytesTotal:Number = 0)
        {
            super(type,bubbles,cancelable);
            this.bytesLoaded = bytesLoaded;
            this.bytesTotal = bytesTotal;
        }

        override public function clone() : Event
        {
            return new ProgressEvent(this.type,this.bubbles,this.cancelable,this.bytesLoaded,this.bytesTotal);
        }

        override public function toString() : String
        {
            return this.formatToString("ProgressEvent","type","bubbles","cancelable","eventPhase","bytesLoaded","bytesTotal");
        }
    }
}
