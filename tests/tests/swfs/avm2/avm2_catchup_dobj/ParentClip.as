package  {
	
	import flash.display.MovieClip;
	
	
	public class ParentClip extends MovieClip {
		
		
		public function ParentClip() {
			trace("Parent clip constructor");
			var self = this;
			this.addEventListener("enterFrame", function(e) {
				trace("enterFrame for ParentClip: this.currentFrame=" + self.currentFrame);
			})
		}
	}
	
}
