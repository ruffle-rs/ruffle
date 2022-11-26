package {
	import flash.errors.IllegalOperationError;
	
	public class Test {
		public function Test() {
		}
	}
}

function foo() {
	throw new ArgumentError("My Error", 50);
}

try {
	foo()
} catch (err) {
	trace("Caught error!");
	trace(err.name);
	trace(err);
}