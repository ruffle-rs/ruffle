package  {
	
	import flash.display.MovieClip;
	import flash.text.TextField;
	
	
	public class SuperClass extends MovieClip {
		public var text: TextField;
		public var box: MovieClip;
		
		public function SuperClass() {
			trace("// In SuperClass constructor");

			trace("// this.text");
			trace(this.text);
			trace("");

			trace("// this.box");
			trace(this.box);
			trace("");

			trace("// this[\"circle\"]");
			trace(this["circle"]);
			trace("");

			trace("");
		}
	}
	
}
