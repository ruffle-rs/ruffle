package {
	public class Test {}
}

class Evil {
	public function get propget() {
		return arguments.callee;
	}
}

function testfunc() {
	trace(arguments.length);
	
	for (var i = 0; i < arguments.length; i += 1) {
		trace(arguments[i]);
	}
}

function argprops() {
	trace("///Array.prototype.test = \"test\";");
	Array.prototype.test = "test";
	
	trace("///arguments.test");
	trace(arguments.test);
	
	trace("///argument.callee === argprops");
	trace(arguments.callee === argprops);
}

trace("///testfunc(\"arg1\");");
testfunc("arg1");

trace("///testfunc(\"arg1\", \"arg2\", \"arg3\");");
testfunc("arg1", "arg2", "arg3");

trace("///testfunc(\"arg1\", \"arg2\", \"arg3\", \"arg4\", \"arg5\");");
testfunc("arg1", "arg2", "arg3", "arg4", "arg5");

argprops();

trace("///(Evil is a class with a property that resolves to it's own getter)");
trace("///var x = new Evil();");
var x = new Evil();

trace("///x.propget");
trace(x.propget);

trace("///x.propget()");
trace(x.propget());

trace("///x.propget === x.propget()");
trace(x.propget === x.propget());