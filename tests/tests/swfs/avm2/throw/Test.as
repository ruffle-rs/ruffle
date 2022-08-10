package {
	public class Test {
		public function Test() {}
	}
}

import flash.errors.IllegalOperationError;

function foo() {
	throw new IllegalOperationError("My Error", 30);
}

try {
	foo()
} catch (err) {
	trace("Caught error!");
	
	// FIXME - we currently don't use the original thrown
	// object in a 'catch' block. Once that's implemented, this
	// statement will start printing the correct values.
	trace(err);
}