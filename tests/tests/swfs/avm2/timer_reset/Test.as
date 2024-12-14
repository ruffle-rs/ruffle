package {
	import flash.display.Sprite;
	import flash.utils.Timer;
	import flash.events.TimerEvent;

	public class Test extends Sprite
	{
		private var interval_id:uint;
		private var interval_cnt:int = 0;
		private var timer:Timer;
		private var timer_cnt:int = 0;

		private function tick_A():void
		{
			// this ticks at every second
			interval_cnt++;
			trace("interval tick", this.interval_cnt);
			// At second 2, the Timer that last fired at 800ms is set so that it should fire again 1100ms from now,
			// so after the 3rd second. Not 1100 ms after it last fired, so not before the 3rd second.
			if (interval_cnt == 2)
				timer.delay = 1100;
		}

		private function timerEvent(e:TimerEvent):void
		{
			timer_cnt++;
			trace("timerEvent", timer_cnt);
		}

		private function timerComplete(e:TimerEvent):void
		{
			timer_cnt++;
			trace("timer complete", timer_cnt);
			flash.utils.clearInterval(this.interval_id);
		}

		public function Test()
		{
			this.interval_id = flash.utils.setInterval(this.tick_A, 1000);
			trace("ID", this.interval_id);

			this.timer = new Timer(800, 3);
			timer.addEventListener(TimerEvent.TIMER, timerEvent);
			timer.addEventListener(TimerEvent.TIMER_COMPLETE, timerComplete);
			timer.start();
		}
	}
}
