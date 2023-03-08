package  {
	
	import flash.display.MovieClip;
	
	
	public class GrandChild extends MovieClip {
		
		private static var counter:uint = 0;
		private var myId:uint;
		
		public function GrandChild() {
			this.myId = GrandChild.counter++;
			var child = this;
			this.addEventListener("enterFrame", function(e) {
				if (MainTimeline.done) {
					return;
				}
				trace("GrandChild child.myId = " + child.myId + " child.parent=" + child.parent + " in enterFrame: child.currentFrame = " + child.currentFrame + " child.isPlaying = " + child.isPlaying);
			})
		
			this.addEventListener("exitFrame", function(e) {
				if (MainTimeline.done) {
					return;
				}
				trace("GrandChild child.myId = " + child.myId + " child.parent=" + child.parent + " in exitFrame: child.currentFrame = " + child.currentFrame + " child.isPlaying = " + child.isPlaying);
			})

			this.addEventListener("frameConstructed", function(e) {
				if (MainTimeline.done) {
					return;
				}
				trace("GrandChild child.myId = " + child.myId + " child.parent=" + child.parent + " in frameConstructed: child.currentFrame = " + child.currentFrame + " child.isPlaying = " + child.isPlaying);
			})			
		}
	}
	
}
