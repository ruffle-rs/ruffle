package {
	import flash.display.MovieClip;
	import flash.events.Event;
	
	public class BaseShape extends MovieClip {
		public function BaseShape() {
			super();
			trace(this.name + ": Constructed");
			trace(this.name + ": Parent");
			trace(this.parent);
			trace(this.name + ": Root");
			trace(this.root);
			trace(this.name + ": Stage");
			trace(this.stage);
		}
	}
}