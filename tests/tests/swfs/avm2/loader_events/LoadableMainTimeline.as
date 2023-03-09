package {
	import flash.display.Stage;
	import flash.display.Sprite;
	import flash.display.MovieClip;
	import flash.events.Event;

	public class LoadableMainTimeline extends MovieClip {
		public function LoadableMainTimeline() {
			trace("Hello from loaded SWF main timeline: this.stage=" + this.stage + " this.parent=" + this.parent);

			var circle:Sprite = new Sprite();
			circle.graphics.beginFill(0xFFCC00);
			circle.graphics.drawCircle(50, 50, 50);
			
			var main = this;
			
			circle.addEventListener(Event.ADDED, function(e) {
				trace("Event.ADDED: circle.parent =" + circle.parent + " circle.stage=" + circle.stage + " main.parent=" + main.parent + " main.stage=" + main.stage)
			});
		
			circle.addEventListener(Event.ADDED_TO_STAGE, function(e) {
				trace("Event.ADDED_TO_STAGE: circle.parent =" + circle.parent + " circle.stage=" + circle.stage + " main.parent=" + main.parent + " main.stage=" + main.stage);
			})

			trace("Adding circle: " + circle);
			this.addChild(circle);
			trace("Added circle");
		}
	}
}