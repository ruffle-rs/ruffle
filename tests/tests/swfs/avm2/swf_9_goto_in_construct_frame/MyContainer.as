package  {
	
	import flash.display.MovieClip;
	import flash.events.MouseEvent;
	import flash.utils.setTimeout;
	
	
	public class MyContainer extends MovieClip {
		
		
		public function MyContainer() {
			var self = this;
			addFrameScript(0,function():*
			 {
				trace("MyContainer framescript 0");
			});
			addFrameScript(1,function():*
			 {
				trace("MyContainer framescript 1");
			 });
			 super();
			 addEventListener("enterFrame",function():*
			 {
				trace("MyContainer enterFrame");
			 });
			 addEventListener("frameConstructed",function():*
			 {
				trace("MyContainer frameConstructed");
				trace("Before MyContainer.gotoAndStop(2)");
				self.gotoAndStop(2);
				trace("After MyContainer.gotoAndStop(2)");
			 });
		}
	}
	
}
