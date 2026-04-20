package flash.events {
    public class SyncEvent extends Event {
        public static const SYNC:String = "sync";

        private var _changeList:Array;

        public function SyncEvent(
            type:String,
            bubbles:Boolean = false,
            cancelable:Boolean = false,
            changeList:Array = null
        ) {
            super(type, bubbles, cancelable);
            this.changeList = changeList;
        }

        public function get changeList():Array {
            return this._changeList;
        }
        public function set changeList(value:Array):void {
            this._changeList = value;
        }

        override public function clone():Event {
            return new SyncEvent(
                this.type,
                this.bubbles,
                this.cancelable,
                this.changeList
            );
        }

        override public function toString():String {
            return this.formatToString(
                "SyncEvent",
                "type",
                "bubbles",
                "cancelable",
                "eventPhase",
                "changeList"
            );
        }
    }
}
