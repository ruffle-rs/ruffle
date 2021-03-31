// #3169: Test that there is no borrow error if a text field binding gets added/removed inside a StageObject setter.

class Main {
	static function main(root) {
		root.stop();
		root.addProperty("foo",
			function(){},
			function(val) {
				trace("foo setter");
				var txt = root.createTextField("txt", 0, 0, 0, 100, 100);
				txt.variable = "blah";
				txt.removeTextField();
			});
			
		root.foo = 1;
		trace("end");
	}
}
