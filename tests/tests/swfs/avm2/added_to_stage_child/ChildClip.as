package  {
	
	import flash.display.MovieClip;
	import flash.events.Event;
	
	public class ChildClip extends MovieClip {

		public function ChildClip() {
			trace("ChildClip constructor");
			this.addEventListener("addedToStage", this.addedToStage);
			this.addEventListener("added", this.added);
		}
	
		private function addedToStage(e:Event) {
			trace("ChildClip event addedToStage: this.parent=" + this.parent + " this.stage=" + this.stage);
		}
	
		private function added(e:Event) {
			trace("ChildClip event added: this.parent=" + this.parent + " this.stage=" + this.stage);
		}
	}
	
}
