package  {
	
	import flash.display.MovieClip;
	
	
	public class ChildClip extends EventWatcher {
		
		
		public function ChildClip() {
			trace("ChildClip constructor!");
			var self = this;
			this.addEventListener("enterFrame", function(e) {
				trace("enterFrame for ChildClip: this.currentFrame=" + self.currentFrame);
			})
		}
	}
	
}
