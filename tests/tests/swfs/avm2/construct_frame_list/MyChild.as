package  {
	
	import flash.display.MovieClip;
	
	
	public class MyChild extends MovieClip {
		
		
		public function MyChild() {
			trace("Constructed MyChild with name: " + this.name);
			if (this.name == "mychild2") {
				trace("Constructing as3child");
				var as3child = new MyChild();
				trace("as3child constructed");
				as3child.name = "as3child";
				this.parent.addChildAt(as3child, 2);
			}
		}
	}
	
}
