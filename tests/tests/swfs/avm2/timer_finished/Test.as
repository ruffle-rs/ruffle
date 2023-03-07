package {
	import flash.events.TimerEvent;
	import flash.utils.Timer;
	import flash.utils.setTimeout;

	public class Test {
		public function Test() {
			var timer = new Timer(100, 2);
			timer.addEventListener(TimerEvent.TIMER, function(e) {
				trace("TimerEvent.TIMER: timer.running=" + timer.running + " timer.currentCount=" + timer.currentCount + " timer.repeatCount=" + timer.repeatCount);
			});
			timer.addEventListener(TimerEvent.TIMER_COMPLETE, function(e) {
				trace("TimerEvent.TIMER_COMPLETE: timer.running=" + timer.running + " timer.currentCount=" + timer.currentCount + " timer.repeatCount=" + timer.repeatCount);
			})
			timer.start();
		
			setTimeout(function() {
				trace("After 300ms: timer.running=" + timer.running + " timer.currentCount=" + timer.currentCount + " timer.repeatCount=" + timer.repeatCount);
				trace("Starting timer again");
				timer.start();
				
				setTimeout(function() {
					trace("After 200ms: timer.running=" + timer.running + " timer.currentCount=" + timer.currentCount + " timer.repeatCount=" + timer.repeatCount);
					trace("Stopping/Starting timer again");
					timer.stop();
					timer.start();
				}, 200);
			}, 300);
		}
	}
}