class ExtendedFurther {
	var test = "ExtendedFurther";
	function ExtendedFurther() {
		super();
		trace("// ExtendedFurther");
		trace(this["test"]);
	}
	
	function test_method() {
		super.test_method();
		trace("// ExtendedFurther.test_method");
		trace(this["test"]);
	}
	
	function get test_property() {
		var tp = super.test_property;
		trace("// ExtendedFurther.test_property (get)");
		trace(this["test"]);
		return tp;
	}
	
	function set test_property(val) {
		super.test_property = val;
		trace("// ExtendedFurther.test_property (set)");
		trace(this["test"]);
	}
}