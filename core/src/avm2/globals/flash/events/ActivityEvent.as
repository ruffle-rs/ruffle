package flash.events {
	public class ActivityEvent extends Event {
		public var activating:Boolean;

		public function ActivityEvent(type:String, bubbles:Boolean = false, cancelable:Boolean = false, activating:Boolean = false) {
			super(type, bubbles, cancelable);
			this.activating = activating;
		}

		override public function clone() : Event {
			return new ActivityEvent(this.type, this.bubbles, this.cancelable, this.activating);
		}

		override public function toString(): String {
			return formatToString("ActivityEvent","type","bubbles","cancelable","eventPhase","activating");
		}
	}
}
