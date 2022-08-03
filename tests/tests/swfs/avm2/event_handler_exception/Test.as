package {
	import flash.events.Event;
	import flash.display.Sprite;
	import flash.display.Stage;
	
	public class Test {
		public function Test(stage: Stage) {
			stage.addEventListener(Event.ENTER_FRAME, function() {
				trace("First listener!");
				throw new Error("Exception in first listener");
			});

			stage.addEventListener(Event.ENTER_FRAME, function() {
				trace("Second listener!");

				var sprite = new Sprite();
				sprite.addEventListener("customEvent", function() {
					trace("First custom event listener!");
					throw new Error("Exception in custom event handler");
				});

				sprite.addEventListener("customEvent", function() {
					trace("Second custom event listener!");
				});

				sprite.dispatchEvent(new Event("customEvent"));	
			});		
		}
	}
}