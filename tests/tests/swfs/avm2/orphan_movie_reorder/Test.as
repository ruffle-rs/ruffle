package {
	import flash.display.DisplayObject;
	import flash.display.MovieClip;
	import flash.utils.setTimeout;
	import flash.utils.Dictionary;

	public class Test {
		public function Test(movie:MovieClip) {
			
			// None of this re-ordering should affect enterFrame, since they're handled
			// by broadcast event listerners, which do not get affected by adding/removing
			// orphans.
			
			var done = false;
			movie.addEventListener("enterFrame", function() {
				if (done) {
					return;
				}
				done = true;
			
				var orphan1 = makeOrphan("1");
				var orphan2 = makeOrphan("2");
				var orphan3 = makeOrphan("3");
				var orphan4 = makeOrphan("4");
				var orphan5 = makeOrphan("5");
				var counter = 0;
			
				movie.addEventListener("enterFrame", function(e) {
					trace("enterFrame: Movie counter=" + counter + " currentFrame=" + movie.currentFrame);
					if (counter == 2) {
						trace("Adding and removing orphan2 as child");
						movie.addChild(orphan2);
						movie.removeChild(orphan2);
					}
				
					if (counter == 5) {
						trace("Adding orphan 3 as child");
						movie.addChild(orphan3);
					}
				
					if (counter == 8) {
						trace("Removing orphan 3 as child");
						movie.removeChild(orphan3);
					}
				
					counter += 1;
				});
			});
		}
	
		private function makeOrphan(name:String): FrameHelper {
			var orphan = new FrameHelper();
			//orphan.play();
			orphan.addEventListener("enterFrame", function(e) {
				trace("enterFrame: Orphan " + name + " in frame: " + orphan.currentFrame);
			});
			for (var i = 0; i <= 10; i++) {
				// Hack to get a closure with its own copy of 'i''
				function wrapper(counter:int) {
					return function() {
						trace("Orphan " + name + " in framescript=" + counter + " currentFrame=" + orphan.currentFrame);
					}
				}
			
				// FIXME - I haven't been able to figure out what determines the order
				// that frame scripts run in on the orphan list. I suspect that Flash
				// is iterating over some internal hash table. For now, only add
				// frame scripts for a single orphan, so that we don't depend on the order.
				if (name == "1") {
					orphan.addFrameScript(i, wrapper(i));
				}
			}
			//trace("Initial frame: " + orphan.currentFrame);
			
			return orphan;
		}
	}
}