package  {
	
	public class ChildClass extends SuperClass {
		public function ChildClass() {
			super();
			trace("// In ChildClass constructor");
			
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
