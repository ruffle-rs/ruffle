package {
	public class Test {
	}
}

import com.ruffle.MyClass;

function someFunc(arg: String) {}

var cls = MyClass;
var func = someFunc;
var method = new MyClass().myMethod;
var singleArgMethod = new MyClass().singleArgMethod;
var varArgMethod = new MyClass().varArgMethod;
var defaultArgs = new MyClass().defaultArgs;
var defaultArgVarargs = new MyClass().defaultArgVarargs;
try {
	new cls(true, "Bad arg");
} catch (e) {
	trace("Caught error: " + e);
}

try {
	func("First", "Bad second", true);
} catch (e) {
	trace("Caught error: " + e);
}


try {
	method("First", "Bad second", true);
} catch (e) {
	trace("Caught error: " + e);
}

try {
	singleArgMethod();
} catch (e) {
	trace("Caught error: " + e);
}

try {
	varArgMethod();
} catch (e) {
	trace("Caught error: " + e);
}

try {
	defaultArgs();
} catch (e) {
	trace("Caught error: " + e);
}

try {
	defaultArgVarargs();
} catch (e) {
	trace("Caught error: " + e);
}