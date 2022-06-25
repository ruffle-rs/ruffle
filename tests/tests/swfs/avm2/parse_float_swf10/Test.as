// compiled with mxmlc -swf-version 10

package {
    public class Test {}
}

// Test no arguments.
trace("// parseFloat()");
trace(parseFloat());

// Test integer.
trace("// parseFloat(\"12345\")");
trace(parseFloat("12345"));

// Test decimal point.
trace("// parseFloat(\"012345.67890\")");
trace(parseFloat("012345.67890"));

// Test that leading/trailing whitespace are allowed.
trace("// parseFloat(\"    99999.99999          \")");
trace(parseFloat(" \t\r\n99999.99999\t\r\n      "));

// Test long numbers (more than 15 digits).
trace("// parseFloat(\"-22222222222222222\")");
trace(parseFloat("-22222222222222222"));

trace("// parseFloat(\"-22222222.222222222\")");
trace(parseFloat("-22222222.222222222"));

// Test subnormal number.
trace("// parseFloat(\".0000000000000000000000005\")");
trace(parseFloat(".0000000000000000000000005"));

// Test that trailing garbage is ignored.
trace("// parseFloat(\"0000.12345GIBBERISH\")");
trace(parseFloat("0000.12345GIBBERISH"));

// Test exponent.
trace("// parseFloat(\"9e99999\")");
trace(parseFloat("9e99999"));

trace("// parseFloat(\"+100e-100\")");
trace(parseFloat("+100e-100"));

trace("// parseFloat(\"-123.234E+66\")");
trace(parseFloat("-123.234E+66"));

trace("// parseFloat(\".2E20E1\")");
trace(parseFloat(".2E20E1"));

trace("// parseFloat(\"-034.1+e20\")");
trace(parseFloat("-034.1+e20"));

trace("// parseFloat(\"10e\")");
trace(parseFloat("10e"));

trace("// parseFloat(\"e10\")");
trace(parseFloat("e10"));

trace("// parseFloat(\"10e-\")");
trace(parseFloat("10e-"));

// Test exponent overflow.
trace("// parseFloat(\"1e4294967297\")");
trace(parseFloat("1e4294967297"));

trace("// parseFloat(\"1e2147483648\")");
trace(parseFloat("1e2147483648"));

trace("// parseFloat(\"1e-2147483648\")");
trace(parseFloat("1e-2147483648"));

// Test multiple dots.
trace("// parseFloat(\"1.2345.678\")");
trace(parseFloat("1.2345.678"));

trace("// parseFloat(\"1.2345.6e50\")");
trace(parseFloat("1.2345.6e50"));

// Test Infinity.
trace("// parseFloat(\"Infinity\")");
trace(parseFloat("Infinity"));

trace("// parseFloat(\"-Infinity\")");
trace(parseFloat("-Infinity"));

trace("// parseFloat(\"+Infinity\")");
trace(parseFloat("+Infinity"));

trace("// parseFloat(\"Infinitya\")");
trace(parseFloat("Infinitya"));

trace("// parseFloat(\"Infinity   a\")");
trace(parseFloat("Infinity   a"));

trace("// parseFloat(\".   Infinity\")");
trace(parseFloat(".   Infinity"));

trace("// parseFloat(\"e10   Infinity\")");
trace(parseFloat("e10   Infinity"));

trace("// parseFloat(\".e10   Infinity\")");
trace(parseFloat(".e10   Infinity"));

trace("// parseFloat(\"1   Infinity\")");
trace(parseFloat("1   Infinity"));

// Test invalid strings.
trace("// parseFloat(\"BADBAD\")");
trace(parseFloat("BADBAD"));

trace("// parseFloat(\"\")");
trace(parseFloat(""));

trace("// parseFloat(\"-\")");
trace(parseFloat("-"));

trace("// parseFloat(\"0xff\")");
trace(parseFloat("0xff"));

trace("// parseFloat(String.fromCharCode(305))");
trace(parseFloat(String.fromCharCode(305)));

// Test non-string inputs.
trace("// parseFloat(true)");
var b = true;
trace(parseFloat(b));

trace("// parseFloat(1.2)");
var f = 1.2;
trace(parseFloat(f));

trace("// parseFloat(Infinity)");
f = Infinity;
trace(parseFloat(f));

trace("// parseFloat({ toString })");
var o = {
    toString: function() { return "5"; }
};
trace(parseFloat(o));

trace("// parseFloat(new ClassWithToString())");
class C {
    public function toString(): String { return "6"; }
}
var c = new C();
trace(parseFloat(c));
