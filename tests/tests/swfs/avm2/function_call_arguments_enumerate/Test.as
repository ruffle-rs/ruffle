package {
	public class Test {
		public function Test() {
			
		}
	}
}

function my_varargs() {
	for (var arg in arguments) {
		trace("Arg: " + arg);
	}
	trace("Callee: " + arguments.callee);
}

function caller() {
	var func = my_varargs;
	func();
	func("a", 1, true);	
}

caller();