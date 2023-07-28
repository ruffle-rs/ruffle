package  {
	
	import flash.display.MovieClip;
	
	
	public class FirstChild extends MovieClip {
		
		
		public function FirstChild() {
			trace("FirstChild: before super() call - this.parent[\"firstChild\"] = " + this.parent["firstChild"]);
			super();
			trace("FirstChild: after super() call - this.parent[\"firstChild\"] = " + this.parent["firstChild"]);
			trace("FirstChild: running gotoAndStop on parent")
			MovieClip(this.parent).gotoAndStop(3);
			trace("FirstChild: ran gotoAndStop on parent")
		}
	}
	
}
