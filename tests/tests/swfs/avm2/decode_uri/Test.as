package {

	public class Test {
	}
}

import flash.utils.getDefinitionByName;

var fns = ["decodeURI", "decodeURIComponent"];
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

	var input = "%3A";
	trace("// " + fnName + "(\"" + input + "\")");
	trace(fn(input));
	trace("");
	
	var input = "%E0%A4%A";
	trace("// " + fnName + "(\"" + input + "\")");
	try {
		trace(fn(input));
	} catch (e) {
		trace(e);
	}
	trace("");
	var input = "%FFabcd";
	trace("// " + fnName + "(\"" + input + "\")");
	try {
		trace(fn(input));
	} catch (e) {
		trace(e);
	}
	trace("");
	
	var src:String = String.fromCharCode(0xD842, 0xDF9F);
	var input = encodeURIComponent(src);
	trace("// " + fnName + "(\"" + input + "\")");
	try {
		trace(fn(input));
	} catch (e) {
		trace(e);
	}
	trace("");
	
	
	var input = "\x05";
	trace("// " + fnName + "(\"\\x05\")");
	trace(fn(input));
	trace("");

	var input = "😭";
	trace("// " + fnName + "(\"" + input + "\")");
	trace(fn(input));
	trace("");

	var input = "~!%40%23%24%25%5E%26*()_%2B%5B%5D%5C%7B%7D%7C%3B'%2C.%2F%3C%3E%3F";
	trace("// " + fnName + "(\"" + input + "\")");
	trace(fn(input));
	trace("");
	
}
