package  {
	
	import flash.display.MovieClip;
	
	
	public class SecondChild extends MovieClip {
		
		
		public function SecondChild() {
			trace("SecondChild: before super() call - this.parent[\"secondChild\"] = " + this.parent["secondChild"]);
			super();
			trace("SecondChild: after super() call - this.parent[\"secondChild\"] = " + this.parent["secondChild"]);
		}
	}
	
}
