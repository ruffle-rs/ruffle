package {
	import flash.display.Stage;
	import flash.display.Sprite;
	import flash.display.MovieClip;
	import flash.events.Event;

	public class Loadable extends MovieClip {
		public function Loadable() {
			trace("Hello from loaded SWF:");
			trace("Loaded swf loaderInfo.url: " + this.urlPrefix(this.loaderInfo.url) + " content: " + this.loaderInfo.content);
			this.addEventListener(Event.ADDED_TO_STAGE, this.onAddedToStage);
			var circle:Sprite = new Sprite();
			circle.graphics.beginFill(0xFFCC00);
			circle.graphics.drawCircle(50, 50, 50);

			this.addChild(circle);
		}
		private function onAddedToStage(event: *) {
			trace("Added to stage: this.loaderInfo.url = " + this.urlPrefix(this.loaderInfo.url) + " this.loaderInfo.content = " + this.loaderInfo.content);
		}
	
		private function urlPrefix(url: String): String {
			return url ? url.substr(0, 8) : url;
		}
	}
}