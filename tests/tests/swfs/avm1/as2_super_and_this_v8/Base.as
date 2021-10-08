class Base {
	function Base() {
		trace("// Base");
		trace(this["test"]);
		trace("// super.__proto__ === undefined");
		trace(super.__proto__ === undefined);
	}

	function test_method() {
		trace("// Base.test_method");
		trace(this["test"]);
		trace("// super.__proto__ === undefined");
		trace(super.__proto__ === undefined);
	}

	function get test_property() {
		trace("// Base.test_property (get)");
		trace(this["test"]);
		trace("// super.__proto__ === undefined");
		trace(super.__proto__ === undefined);
		return "test property";
	}

	function set test_property(val) {
		trace("// Base.test_property (set)");
		trace(this["test"]);
		trace("// super.__proto__ === undefined");
		trace(super.__proto__ === undefined);
	}
}
