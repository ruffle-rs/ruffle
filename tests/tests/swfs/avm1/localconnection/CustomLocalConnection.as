class CustomLocalConnection extends LocalConnection {
	function test() {
		trace("custom.test was called with " + arguments.length + " argument" + (arguments.length == 0 ? "" : "s"));
		if (arguments.length > 0) {
			trace("  " + repr(arguments));
		}
	}

	function throwAnError() {
		trace("custom.throwAnError was called");
		//throw "aah!"; // [NA] this crashes every Flash Player I've tried
		//throw {}; // [NA] this causes an error when constructing the AsyncErrorEvent
		//throw new Error("aaah!");
	}
}