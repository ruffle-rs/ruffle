class Extended extends Base {
	var test = "Extended";

	function Extended() {
		super();
		trace("// Extended");
		trace(this["test"]);
		trace("// super.__proto__ === Object.prototype");
		trace(super.__proto__ === Object.prototype);
	}

	function test_method() {
		super.test_method();
		trace("// Extended.test_method");
		trace(this["test"]);
		trace("// super.__proto__ === Object.prototype");
		trace(super.__proto__ === Object.prototype);
	}

	function get test_property() {
		var tp = super.test_property;
		trace("// Extended.test_property (get)");
		trace(this["test"]);
		trace("// super.__proto__ === Object.prototype");
		trace(super.__proto__ === Object.prototype);
		return tp;
	}

	function set test_property(val) {
		super.test_property = val;
		trace("// Extended.test_property (set)");
		trace(this["test"]);
		trace("// super.__proto__ === Object.prototype");
		trace(super.__proto__ === Object.prototype);
	}
}
