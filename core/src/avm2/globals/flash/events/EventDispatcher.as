// This is a stub - the actual class is defined in `eventdispatcher.rs`
package flash.events {
	public class EventDispatcher implements IEventDispatcher {
		internal var _target:IEventDispatcher;
		internal var _dispatchList:Object;

		public function EventDispatcher(target:IEventDispatcher = null) {
			this._target = target;
		}

		public native function addEventListener(type:String, listener:Function, useCapture:Boolean = false, priority:int = 0, useWeakReference:Boolean = false):void;
		public native function removeEventListener(type:String, listener:Function, useCapture:Boolean = false):void;
		public native function dispatchEvent(event:Event):Boolean;
		public native function hasEventListener(type:String):Boolean;
		public native function willTrigger(type:String):Boolean;

		public native function toString():String;
	}
}
