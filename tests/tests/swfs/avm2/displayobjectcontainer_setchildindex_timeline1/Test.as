package  {
	
	import flash.display.MovieClip;
	import flash.system.fscommand;
	
	public class Test extends MovieClip {
		
		var tick = 0;

		public function Test() {
			setChildIndex(blue, this.numChildren - 1);
			addFrameScript(0, enterFrame, 1, enterFrame);
		}
		
		private function enterFrame() {
			for (var i = 0; i < numChildren; i++) {
				var child = getChildAt(i);
				var name = child.name ? " (" + child.name + ")" : "";
				trace(child + name);
			}
			trace("");
			tick++;
			if (tick == 4) {
				fscommand("quit");
			}
		}
	}
	
}
