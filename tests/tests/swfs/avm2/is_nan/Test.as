package {
	public class Test {
	}
}

trace("isNaN(true)");
trace(isNaN(true));
trace("isNaN(false)");
trace(isNaN(false));
trace("isNaN(10.0)");
trace(isNaN(10.0));
trace("isNaN(-10.0)");
trace(isNaN(-10.0));
trace("isNaN(0.0)");
trace(isNaN(0.0));
trace("isNaN(NaN)");
trace(isNaN(NaN));
trace("isNaN(Infinity)");
trace(isNaN(Infinity));
trace("isNaN(-Infinity)");
trace(isNaN(-Infinity));
trace("isNaN(\"\")");
trace(isNaN(""));
trace("isNaN(\"hello\")");
trace(isNaN("hello"));
trace("isNaN(\" \")");
trace(isNaN(" "));
trace("isNaN(\"  5  \")");
trace(isNaN("  5  "));
trace("isNaN(\"0\")");
trace(isNaN("0"));
trace("isNaN(\"NaN\")");
trace(isNaN("NaN"));
trace("isNaN(\"Infinity\")");
trace(isNaN("Infinity"));
trace("isNaN(\"-Infinity\")");
trace(isNaN("-Infinity"));
trace("isNaN(\"100a\")");
trace(isNaN("100a"));
trace("isNaN(\"0x10\")");
trace(isNaN("0x10"));
trace("isNaN(\"0xhello\")");
trace(isNaN("0xhello"));
trace("isNaN(\"0x1999999981ffffff\")");
trace(isNaN("0x1999999981ffffff"));
trace("isNaN(\"0xUIXUIDFKHJDF012345678\")");
trace(isNaN("0xUIXUIDFKHJDF012345678"));
trace("isNaN(\"123e-1\")");
trace(isNaN("123e-1"));
trace("isNaN()");
trace(isNaN());
