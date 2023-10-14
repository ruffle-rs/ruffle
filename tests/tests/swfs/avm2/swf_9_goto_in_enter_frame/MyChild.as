package  {
	
	import flash.display.MovieClip;
	
	
	public class MyChild extends MovieClip {
		
		
		public function MyChild() {
			this.addFrameScript(0, function() {
				trace("MyChild framescript 0");
			});
			this.addFrameScript(1, function() {
				trace("MyChild framescript 1");
			});
			trace("Calling MyChild super");
			super();
			trace("Called MyChild super");
		}
	}
	
}
