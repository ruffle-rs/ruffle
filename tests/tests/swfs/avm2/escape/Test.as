package {

	public class Test {
	}
}

import flash.utils.getDefinitionByName;

var fns = ["escape", "encodeURI", "encodeURIComponent"];
for each (var fnName in fns) {
	var fn = getDefinitionByName(fnName);
	trace("// " + fnName + "()");
	trace(fn());
	trace("");

	trace("// " + fnName + "(undefined)");
	trace(fn(undefined));
	trace("");

	trace("// typeof(" + fnName + "(undefined))");
	trace(typeof(fn(undefined)));
	trace("");

	trace("// " + fnName + "(null)");
	trace(fn(null));
	trace("");

	var input = "test";
	trace("// " + fnName + "(\"" + input + "\")");
	trace(fn(input));
	trace("");

	var input = "!\"£$%^&*()1234567890qwertyuiop[]asdfghjkl;'#\zxcvbnm,./QWERTYUIOP{}ASDFGHJKL:@~|ZXCVBNM<>?\u0010";
	trace("// " + fnName + "(\"" + input + "\")");
	trace(fn(input));
	trace("");

	var input = "\x05";
	trace("// " + fnName + "(\"\\x05\")");
	trace(fn(input));
	trace("");

	var input = "😭";
	trace("// " + fnName + "(\"" + input + "\")");
	trace(fn(input));
	trace("");	
}
