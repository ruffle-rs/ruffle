package {
	public class Superclass {

		public var superField:Number = 20;
		
		public function myMethod(arg1: String, arg2: Boolean) {
			trace("In superclass myMethod: " + arg1 + " " + arg2);
		}
	
		public function get myGetter():String {
			trace("Calling superclass getter");
			return "Value from superclass"
		}
	
		public function set mySetter(val: String):void {
			trace("Calling superclass getter with " + val);
		}
	}
}