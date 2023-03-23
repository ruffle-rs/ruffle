package {
	public class Test {}
}

function testfunc(v1, v2, v3) {
	trace(v1);
	trace(v2);
	trace(v3);
}

trace('// testfunc("arg1", "arg2", "arg3");');
testfunc("arg1", "arg2", "arg3");
trace("");

// Errors
var o = undefined;
trace('// o();');
try {
	o();
} catch(e:*) {
	trace("Error: " + e.errorID);
}

trace('// o.foo();');
o = {};
try {
	o.foo();
} catch(e:*) {
	trace("Error: " + e.errorID);
}
trace("");


trace('// o["foo"]();');
try {
	o["foo"]();
} catch(e:*) {
	trace("Error: " + e.errorID);
}
