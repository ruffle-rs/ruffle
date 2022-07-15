package {

	public class Test {
	}
}

trace("// escape()");
trace(escape());
trace("");

trace("// escape(undefined)");
trace(escape(undefined));
trace("");

trace("// typeof(escape(undefined))");
trace(typeof(escape(undefined)));
trace("");

trace("// escape(null)");
trace(escape(null));
trace("");

var input = "test";
trace("// escape(\"" + input + "\")");
trace(escape(input));
trace("");

var input = "!\"£$%^&*()1234567890qwertyuiop[]asdfghjkl;'#\zxcvbnm,./QWERTYUIOP{}ASDFGHJKL:@~|ZXCVBNM<>?\u0010";
trace("// escape(\"" + input + "\")");
trace(escape(input));
trace("");

var input = "\x05";
trace("// escape(\"\\x05\")");
trace(escape(input));
trace("");

var input = "😭";
trace("// escape(\"" + input + "\")");
trace(escape(input));
trace("");
