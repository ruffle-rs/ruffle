package  {
	
	import flash.display.MovieClip;
	
	
	public class Parent extends MovieClip {
		
		public var child_obj:MovieClip;
		
	
		public function Parent() {
			this.buttonMode = true;
			this.addEventListener("mouseDown", function(e) {
				trace("Mousedown on " + e.target.name + " at: " + e.stageX + " " + e.stageY);
 			});
		}
	}
	
}
