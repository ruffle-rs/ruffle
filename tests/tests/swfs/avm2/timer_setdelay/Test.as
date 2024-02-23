package
{
	import flash.display.Sprite;
	import flash.utils.Timer;
	import flash.events.TimerEvent;

	public class Test extends Sprite
	{
		private var timer:Timer;
		private var cnt:int = 0;

		public function Test()
		{
			trace("Starting");
			timer = new Timer(500, 3);
			timer.addEventListener(TimerEvent.TIMER, timerEvent);
			timer.addEventListener(TimerEvent.TIMER_COMPLETE, timerComplete);
			timer.start();
		}

		private function timerEvent(e:TimerEvent):void
		{
			cnt++;
			trace("timerEvent", cnt);
			timer.delay = 500;
		}

		private function timerComplete(e:TimerEvent):void
		{
			trace("timerComplete", cnt);
		}
	}
}
