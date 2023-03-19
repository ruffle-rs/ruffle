package flash.events {
	import __ruffle__.stub_method;
	public class TimerEvent extends Event{
		public static const TIMER:String = "timer";
		public static const TIMER_COMPLETE:String = "timerComplete";

		public function TimerEvent(type:String, bubbles:Boolean = false, cancelable:Boolean= false) {
			super(type, bubbles, cancelable);
		}
		
		override public function clone() : Event {
			return new TimerEvent(this.type,this.bubbles,this.cancelable);
		}

		public native function updateAfterEvent():void;
	}
}
