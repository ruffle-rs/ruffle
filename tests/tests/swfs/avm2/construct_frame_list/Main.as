package  {
	
	import flash.display.MovieClip;
	
	
	public class Main extends MovieClip {
		
		
		public function Main() {
			trace("Main constructor");
			this.addEventListener("enterFrame", function(e) {
				trace("Enter frame!");
			});
			super();
			trace("Main after super");
		}
	}
	
}
