package  {
	
	import flash.display.MovieClip;
	
	
	public class MyChild extends MovieClip {
		
		private static var counter:uint = 0;
		private var myId:uint;
		
		private var grandChild:GrandChild;
		
		public function MyChild(overrideName:String = null, printConstructorCall:Boolean = true) {
			this.myId = MyChild.counter++;
			if (overrideName != null) {
				this.name = overrideName;
			}
			if (printConstructorCall) {
				trace("Running child constructor: this.myId = " + this.myId + " this.parent=" + this.parent);
			}
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
		
			this.grandChild = new GrandChild("manualGrandchild", printConstructorCall);
		}
	}
	
}
