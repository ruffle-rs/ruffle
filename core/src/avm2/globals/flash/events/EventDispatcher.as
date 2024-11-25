package flash.events {
    public class EventDispatcher implements IEventDispatcher {
        [Ruffle(InternalSlot)]
        private var target:IEventDispatcher;

        [Ruffle(InternalSlot)]
        private var dispatchList:Object;

        public function EventDispatcher(target:IEventDispatcher = null) {
            this.target = target;
        }

        public native function addEventListener(type:String, listener:Function, useCapture:Boolean = false, priority:int = 0, useWeakReference:Boolean = false):void;
        public native function removeEventListener(type:String, listener:Function, useCapture:Boolean = false):void;
        public native function dispatchEvent(event:Event):Boolean;
        public native function hasEventListener(type:String):Boolean;
        public native function willTrigger(type:String):Boolean;

        public function toString():String {
            return Object.prototype.toString.call(this);
        }
    }
}
