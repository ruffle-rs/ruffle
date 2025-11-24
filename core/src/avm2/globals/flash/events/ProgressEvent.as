package flash.events {
    public class ProgressEvent extends Event {
        public static const PROGRESS:String = "progress";
        public static const SOCKET_DATA:String = "socketData";

        private var _bytesLoaded:Number;
        private var _bytesTotal:Number;

        public function ProgressEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            bytesLoaded:Number = 0,
            bytesTotal:Number = 0
        ) {
            super(type, bubbles, cancelable);
            this._bytesLoaded = bytesLoaded;
            this._bytesTotal = bytesTotal;
        }

        public function get bytesLoaded():Number {
            return this._bytesLoaded;
        }
        public function set bytesLoaded(value:Number):void {
            this._bytesLoaded = value;
        }

        public function get bytesTotal():Number {
            return this._bytesTotal;
        }
        public function set bytesTotal(value:Number):void {
            this._bytesTotal = value;
        }

        override public function clone():Event {
            return new ProgressEvent(
                this.type,
                this.bubbles,
                this.cancelable,
                this.bytesLoaded,
                this.bytesTotal
            );
        }

        override public function toString():String {
            return this.formatToString(
                "ProgressEvent",
                "type",
                "bubbles",
                "cancelable",
                "eventPhase",
                "bytesLoaded",
                "bytesTotal"
            );
        }
    }
}
