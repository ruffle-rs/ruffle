package {
	public class Test {
	}
}

trace("isFinite(true)");
trace(isFinite(true));
trace("isFinite(false)");
trace(isFinite(false));
trace("isFinite(10.0)");
trace(isFinite(10.0));
trace("isFinite(-10.0)");
trace(isFinite(-10.0));
trace("isFinite(0.0)");
trace(isFinite(0.0));
trace("isFinite(NaN)");
trace(isFinite(NaN));
trace("isFinite(Infinity)");
trace(isFinite(Infinity));
trace("isFinite(-Infinity)");
trace(isFinite(-Infinity));
trace("isFinite(\"\")");
trace(isFinite(""));
trace("isFinite(\"hello\")");
trace(isFinite("hello"));
trace("isFinite(\" \")");
trace(isFinite(" "));
trace("isFinite(\"  5  \")");
trace(isFinite("  5  "));
trace("isFinite(\"0\")");
trace(isFinite("0"));
trace("isFinite(\"NaN\")");
trace(isFinite("NaN"));
trace("isFinite(\"Infinity\")");
trace(isFinite("Infinity"));
trace("isFinite(\"-Infinity\")");
trace(isFinite("-Infinity"));
trace("isFinite(\"100a\")");
trace(isFinite("100a"));
trace("isFinite(\"0x10\")");
trace(isFinite("0x10"));
trace("isFinite(\"0xhello\")");
trace(isFinite("0xhello"));
trace("isFinite(\"0x1999999981ffffff\")");
trace(isFinite("0x1999999981ffffff"));
trace("isFinite(\"0xUIXUIDFKHJDF012345678\")");
trace(isFinite("0xUIXUIDFKHJDF012345678"));
trace("isFinite(\"123e-1\")");
trace(isFinite("123e-1"));
trace("isFinite()");
trace(isFinite());
