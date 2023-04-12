package {
	public class Subclass extends Superclass {

		public var subclassField:String = "Val";
		
		public function Subclass() {
			this.myMethod("First arg", true);
			
			trace("this.myGetter: " + this.myGetter);
			trace("super.myGetter: " + super.myGetter);
			
			trace("super.superField: " + super.superField);
			
			var obj = super;
			trace("obj.myGetter: " + obj.myGetter);
			
			this.mySetter = "setting_on_this";
			super.mySetter = "setting_on_super";
		}
		public override function myMethod(arg1: String, arg2: Boolean) {
			trace("In subclass myMethod: " + arg1 + " " + arg2);
			super.myMethod("direct_arg", true);
			super.myMethod.apply(this, ["apply_arg", true]);
			
		}
	
		public override function get myGetter():String {
			trace("Calling subclass getter");
			return "Value from subclass"
		}
	
		public override function set mySetter(val: String):void {
			trace("Calling subclass getter with " + val);
		}
	}
}