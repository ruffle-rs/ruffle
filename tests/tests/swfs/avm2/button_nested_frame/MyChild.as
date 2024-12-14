package  {
	
	import flash.display.MovieClip;
	
	
	public class MyChild extends MovieClip {
		
		public var myField: uint;
		public var myFieldWithInit: Object = "Default value";
		
		public function MyChild() {
			trace("Calling MyChild " + this.name + " super()");
			super();
			trace("Called MyChild " + this.name + " super()");
		}
	}
	
}
