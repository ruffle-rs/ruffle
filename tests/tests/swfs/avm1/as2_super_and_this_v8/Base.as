class Base {
	function Base() {
		trace("// Base");
		trace(this["test"]);
	}
	
	function test_method() {
		trace("// Base.test_method");
		trace(this["test"]);
	}
	
	function get test_property() {
		trace("// Base.test_property (get)");
		trace(this["test"]);
		return "test property";
	}
	
	function set test_property(val) {
		trace("// Base.test_property (set)");
		trace(this["test"]);
	}
}