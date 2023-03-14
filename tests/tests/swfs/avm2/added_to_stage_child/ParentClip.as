package  {
	
	import flash.display.MovieClip;
	import flash.events.Event;
	
	public class ParentClip extends MovieClip {
		
		
		public function ParentClip() {
			trace("ParentClip constructor");
			this.addEventListener("addedToStage", this.addedToStage);
			this.addEventListener("added", this.added);
		}
	
		private function addedToStage(e:Event) {
			trace("ParentClip event addedToStage: this.parent=" + this.parent + " this.stage=" + this.stage);
		}
	
		private function added(e:Event) {
			trace("ParentClip event added: this.parent=" + this.parent + " this.stage=" + this.stage);
		}
	}
	
}
