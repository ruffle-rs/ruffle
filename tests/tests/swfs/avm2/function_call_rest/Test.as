package {
	public class Test {}
}

function testrest(arg0, arg1, ...rest) {
	trace(arg0);
	trace(arg1);
	
	trace("///(contents of rest...)");
	trace(rest.length);
	
	for (var i = 0; i < rest.length; i += 1) {
		trace(rest[i]);
	}
}

function restprops(...rest) {
	trace("///Array.prototype.test = \"test\";");
	Array.prototype.test = "test";
	
	trace("///rest.test");
	trace(rest.test);
}

trace("///testrest(\"arg1\");");
testrest("arg1");

trace("///testrest(\"arg1\", \"arg2\", \"arg3\");");
testrest("arg1", "arg2", "arg3");

trace("///testrest(\"arg1\", \"arg2\", \"arg3\", \"arg4\", \"arg5\");");
testrest("arg1", "arg2", "arg3", "arg4", "arg5");

restprops();