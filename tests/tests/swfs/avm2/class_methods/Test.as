package {
	public class Test {
	}
}

class Test2 {
	{
		trace("Class constructor");
	}
	
	function Test2() {
		trace("Instance constructor");
	}
	
	static function classMethod() {
		trace("Class method");
	}
	
	function method() {
		trace("Instance method");
	}
}

trace("Script initializer");
Test2.classMethod();
var x = new Test2();
x.method();