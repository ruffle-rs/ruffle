
package {
	import flash.display.MovieClip;

	public class Test {
		private var parentClip:ParentClip;
		public function Test(main: MovieClip) {
			var self = this;
			
			main.stage.addEventListener("mouseDown", function(e) {
				trace("Constructing ParentClip");
				self.parentClip = new ParentClip();
				trace("Constructed ParentClip");
				
				trace("parentClip.currentFrame: " + parentClip.currentFrame);
				trace("parentClip.getChildAt(0): " + parentClip.getChildAt(0));
				trace("parentClip.getChildAt(0).currentFrame: " + MovieClip(parentClip.getChildAt(0)).currentFrame);
				trace("parentClip.getChildAt(0).getChildAt(0): " + MovieClip(parentClip.getChildAt(0)).getChildAt(0));
				trace("parentClip.getChildAt(0).getChildAt(0).currentFrame: " + MovieClip(MovieClip(parentClip.getChildAt(0)).getChildAt(0)).currentFrame);
				
				trace("Adding child");
				main.addChild(parentClip);
				trace("Added child");
				
				trace("Adding other child");
				var watcher1 = new WatcherChild();
				watcher1.name = "Watcher1";
				parentClip.addChild(watcher1);
				trace("Added other child");
				
				trace("Adding additional child");
				var watcher2 = new WatcherChild();
				watcher2.name = "Watcher2";
				watcher1.addChild(watcher2);
				trace("Added other child");
				
				trace("Adding final orphan");
				var watcher3 = new WatcherChild();
				watcher3.name = "Watcher3";
				watcher1.addChild(watcher3);
				watcher1.removeChild(watcher3);
				trace("Added final child");
			});
	
		}
	}
}