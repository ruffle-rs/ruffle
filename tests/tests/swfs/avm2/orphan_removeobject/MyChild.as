package  {
	
	import flash.display.MovieClip;
	
	
	public class MyChild extends MovieClip {
		
		private static var counter:uint = 0;
		private var myId:uint;
		
		private var grandChild:GrandChild;
		
		public function MyChild() {
			this.myId = MyChild.counter++;
			var child = this;
			
			this.addEventListener("enterFrame", function(e) {
				if (MainTimeline.done) {
					return;
				}
				trace("Child " + child.name + " child.myId = " + child.myId + " child.parent=" + child.parent + " in enterFrame: child.currentFrame = " + child.currentFrame + " child.isPlaying = " + child.isPlaying);
			})
		
			this.addEventListener("exitFrame", function(e) {
				if (MainTimeline.done) {
					return;
				}
				trace("Child " + child.name + " child.myId = " + child.myId + " child.parent=" + child.parent + " in exitFrame: child.currentFrame = " + child.currentFrame + " child.isPlaying = " + child.isPlaying);
			})

			this.addEventListener("frameConstructed", function(e) {
				if (MainTimeline.done) {
					return;
				}
				trace("Child " + child.name + " child.myId = " + child.myId + " child.parent=" + child.parent + " in frameConstructed: child.currentFrame = " + child.currentFrame + " child.isPlaying = " + child.isPlaying);
			});
		
			this.grandChild = new GrandChild();
			
		}
	}
	
}
