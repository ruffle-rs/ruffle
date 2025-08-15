package flash.events {
    public class EventDispatcher implements IEventDispatcher {
        [Ruffle(NativeAccessible)]
        private var target:IEventDispatcher;

        [Ruffle(NativeAccessible)]
        private var dispatchList:Object;

        public function EventDispatcher(target:IEventDispatcher = null) {
            this.target = target;
        }

        public native function addEventListener(type:String, listener:Function, useCapture:Boolean = false, priority:int = 0, useWeakReference:Boolean = false):void;
        public native function removeEventListener(type:String, listener:Function, useCapture:Boolean = false):void;
        public native function hasEventListener(type:String):Boolean;

        [Ruffle(NativeCallable)]
        public function dispatchEvent(event:Event):Boolean {
            // Some SWFs rely on the getter for `target` being called
            if (event.target) {
                return this.dispatchEventInternal(event.clone());
            } else {
                return this.dispatchEventInternal(event);
            }
        }

        private native function dispatchEventInternal(event:Event):Boolean;

        public native function willTrigger(type:String):Boolean;

        public function toString():String {
            return Object.prototype.toString.call(this);
        }
    }
}
