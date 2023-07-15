package {
	public class Test {
		public function Test() {
			
		}
	}
}

function returnInt(param: *):int {
	return param;
}

function returnBool(param: *):Boolean {
	return param;
}

function returnNumber(param: *):Number {
	return param;
}

function returnString(param: *):String {
	return param;
}

function returnMyClass(param: *):MyClass {
	return param;
}

for each (var val in [1.0, true, false, null, undefined, "Hello", new MyClass(), new MyOtherClass()]) {
	trace("returnInt(" + val + ") = " + returnInt(val));
	trace("returnBool(" + val + ") = " + returnBool(val));
	trace("returnNumber(" + val + ") = " + returnNumber(val));
	trace("returnString(" + val + ") = " + returnString(val));
	try {
		trace("returnMyClass(" + val + ") = " + returnMyClass(val));
	} catch (e) {
		trace("returnMyClass(" + val + ") threw error: " + e);
	}
}