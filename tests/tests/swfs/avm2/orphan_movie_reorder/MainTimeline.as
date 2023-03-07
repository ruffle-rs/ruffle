package  {
	
	import flash.display.MovieClip;
	
	
	public class MainTimeline extends MovieClip {
		
		private var immediateOrphan:FrameHelper;
		private var frameScriptDone:Boolean = false;
		
		public function MainTimeline() {
			trace("In MainTimeline constructor");
			new Test(this);
			
			var mainTimeline = this;
			this.immediateOrphan = new FrameHelper();
			
			this.immediateOrphan.addFrameScript(0, function() {
				// Only run this once - we have other framescripts in 'Test',
				// and the relative order in which framescripts run seems to
				// be pseudorandom (probably based on the iteration order of some
				// internal hashset)
				if (mainTimeline.frameScriptDone) {
					return;
				}
				mainTimeline.frameScriptDone = true;
				trace("Running immediate orphan frame script: mainTimeline.immediateOrphan.currentFrame = " + mainTimeline.immediateOrphan.currentFrame);
			});
		
			mainTimeline.immediateOrphan.addEventListener("enterFrame", function(e) {
				trace("Running immediateOrphan enterFrame: immediateOrphan.currentFrame = " + immediateOrphan.currentFrame);
			})
			trace("Finished MainTimeline constructor");
		}
	}
	
}
