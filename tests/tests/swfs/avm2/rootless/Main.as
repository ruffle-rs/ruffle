package  {
	
	import flash.display.MovieClip;
	import flash.display.Stage;
	import flash.utils.setTimeout;
	
	
	public class Main extends MovieClip {
		var actualStage: Stage;
		
		function dump(category: String) {
			trace("/// " + category);
			trace("// this");
			trace(this);
			trace("");
			trace("// stage");
			trace(stage);
			trace("");
			trace("// this.actualStage");
			trace(this.actualStage);
			trace("");
			trace("");
		}
		
		
		public function Main() {
			actualStage = stage;
			
			dump("Initial state");
			
			stage.removeChildAt(0);
			dump("Removed root");
			
			setTimeout(reattach, 0);
		}
		
		public function reattach() {
			dump("Timer called");
			
			actualStage.addChild(this);
			dump("Attached root");
		}
	}
	
}
