class Extended extends Base {
	var test = "Extended";
	function Extended() {
		super();
		trace("// Extended");
		trace(this["test"]);
	}
}