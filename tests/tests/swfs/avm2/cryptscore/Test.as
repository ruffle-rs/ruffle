package  {
	
	import flash.display.MovieClip;
	import flash.events.Event;
	
	
	
	public class Test extends MovieClip {
	
		
		public function Test() {
			var score = new CryptScore();
			for (var i = 0; i < 10000; i ++) {
				if (i % 1000 == 0) {
					trace("i = " + i);
				}
				score.value = score.value;
				var value = score.value;
				if (value !== 0) {
					trace("i = " + i + " Bad value: " + value);
				}
			}
		    trace("Done!");
		}

	}
	
}
