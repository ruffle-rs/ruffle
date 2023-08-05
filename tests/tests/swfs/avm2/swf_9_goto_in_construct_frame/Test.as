package  {
	
	import flash.display.MovieClip;
	
	
	public class Test extends MovieClip {
		
      public function Test()
      {
		  trace("Constructing Test");
		  var self = this;
		  this.addFrameScript(1, function() {
			  trace("Stopping Test at frame 2");
			  self.stop();
		  })
      }
	}
	
}
