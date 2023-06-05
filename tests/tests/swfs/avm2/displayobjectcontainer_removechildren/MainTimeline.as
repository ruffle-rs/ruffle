package  {
	
	import flash.display.MovieClip;
	
	
	public class MainTimeline extends MovieClip {
		
		
		public function MainTimeline() {
			trace("Children: " + this.numChildren);
			for (var i = 0; i < this.numChildren; i++) {
				trace("Initializing: " + this.getChildAt(i));
				Helper.framescriptCounts[this.getChildAt(i)] = [];
			}
		
			var self = this;
			
			this.addEventListener("enterFrame", function(e) {
				if (self.currentFrame == 6) {
					self.stop();
					trace("Framescript counts:");
					for (var key in Helper.framescriptCounts) {
						trace(key);
						for each (var line in Helper.framescriptCounts[key]) {
							trace(line);
						}
						trace();
					}
				} 
			})
		}
	}
	
}
