package {
	import flash.display.DisplayObjectContainer;
	import flash.display.MovieClip;
	import flash.utils.setTimeout;

	public class Test {
		public function Test(movie:DisplayObjectContainer) {
			var orphan1 = new FrameHelper();
			orphan1.play();
			orphan1.addEventListener("enterFrame", function() {
				trace("Orphan1 frame: " + orphan1.currentFrame + " isPlaying: " + orphan1.isPlaying + " observing orphan2: " + orphan2.currentFrame);
				
				if (orphan2.currentFrame == 2) {
					trace('Adding additional orphan3');
					var orphan3 = new FrameHelper();
					orphan3.addEventListener("enterFrame", function() {
						trace("Orphan3 frame: " + orphan3.currentFrame);
						
						if (orphan3.currentFrame == 3) {
							trace('Adding and removing orphan3 as a child');
							movie.addChild(orphan3);
							movie.removeChild(orphan3);
						}
					})
				}
			})
		
			var child = new FrameHelper();
			child.addEventListener("enterFrame", function() {
				// FIXME - child.currentFrame is one behind
				trace("Child isPlaying: " + child.isPlaying + " observing orphan2: " + orphan2.currentFrame);

				if (orphan2.currentFrame == 2) {
					trace('Adding additional orphan4');
					var orphan4 = new FrameHelper();
					orphan4.addEventListener("enterFrame", function() {
						trace("Orphan4 frame: " + orphan4.currentFrame);
					})
				}
				
				if (orphan1.currentFrame == 4) {
					trace("Adding orphan1 as child");
					child.addChild(orphan1);
				}
			})
		
			var orphan2 = new FrameHelper();
			var isStopped = false;
		
			orphan2.addEventListener("enterFrame", function() {
				trace("Orphan2 frame: " + orphan2.currentFrame + " isPlaying: " + orphan2.isPlaying);

				if (orphan2.currentFrame == 2) {
					trace('Adding additional orphan5');
					var orphan5 = new FrameHelper();
					orphan5.addEventListener("enterFrame", function() {
						trace("Orphan5 frame: " + orphan5.currentFrame);
					})
				}				
				
				if (orphan2.currentFrame == 3 && !isStopped) {
					trace("Stopping orphan2 at frame 3");
					orphan2.stop();
					isStopped = true;
				}
			})
		
			movie.addChild(child);
	
		}
	}
}