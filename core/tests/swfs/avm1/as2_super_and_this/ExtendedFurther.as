class ExtendedFurther extends Extended {
	var test = "ExtendedFurther";
	function ExtendedFurther() {
		super();
		trace("// ExtendedFurther");
		trace(this["test"]);
	}
}