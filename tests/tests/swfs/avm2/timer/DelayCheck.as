package {
	public class DelayCheck {
		import flash.utils.getTimer;
		private var prev:int;
		public function DelayCheck() {
			this.prev = -1;
		}
	
		public function check(delay:Number) {
			var cur: int = getTimer();
			// Don't check the first call, as the time from the timer
			// start to the *first* tick event seems to be less consistent
			// than the time *between* tick events.
			if (this.prev == -1) {
				this.prev = cur;
				return;
			}
			if ((cur - this.prev) < delay) {
				trace("ERROR: Timer fired too quickly");
				trace("Expected a delay of at least " + delay + " but got " + (cur - prev));
			}
			this.prev = cur;
		}
	}
}
