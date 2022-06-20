package {

	public class Test {
	}
}

trace("// escape()");
trace(escape());
trace("");

var input = "test";
trace("// escape(\"" + input + "\")");
trace(escape(input));
trace("");

var input = "!\"£$%^&*()1234567890qwertyuiop[]asdfghjkl;'#\zxcvbnm,./QWERTYUIOP{}ASDFGHJKL:@~|ZXCVBNM<>?\u0010";
trace("// escape(\"" + input + "\")");
trace(escape(input));
trace("");
