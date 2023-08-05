package  {
	
	import flash.display.MovieClip;
	
	
	public class MyChild extends MovieClip {
		
		
		public function MyChild() {
			trace("Calling MyChild super");
			super();
			trace("Called MyChild super");
			this.addFrameScript(0, function() {
				trace("MyChild framescript 0");
			});
			this.addFrameScript(1, function() {
				trace("MyChild framescript 1");
			});
		}
	}
	
}
