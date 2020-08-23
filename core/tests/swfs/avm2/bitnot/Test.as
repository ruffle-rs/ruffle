package {
	public class Test {
	}
}

function assert_bitwise_not(val) {
	trace(~val);
}

trace("//~true");
assert_bitwise_not(true);

trace("//~false");
assert_bitwise_not(false);

trace("//~null");
assert_bitwise_not(null);

trace("//~undefined");
assert_bitwise_not(undefined);

trace("//~\"\"");
assert_bitwise_not("");

trace("//~\"str\"");
assert_bitwise_not("str");

trace("//~\"true\"");
assert_bitwise_not("true");

trace("//~\"false\"");
assert_bitwise_not("false");

trace("//~0.0");
assert_bitwise_not(0.0);

trace("//~NaN");
assert_bitwise_not(NaN);

trace("//~-0.0");
assert_bitwise_not(-0.0);

trace("//~Infinity");
assert_bitwise_not(Infinity);

trace("//~1.0");
assert_bitwise_not(1.0);

trace("//~-1.0");
assert_bitwise_not(-1.0);

trace("//~0xFF1306");
assert_bitwise_not(0xFF1306);

trace("//~new Object()");
assert_bitwise_not({});

trace("//~\"0.0\"");
assert_bitwise_not("0.0");

trace("//~\"NaN\"");
assert_bitwise_not("NaN");

trace("//~\"-0.0\"");
assert_bitwise_not("-0.0");

trace("//~\"Infinity\"");
assert_bitwise_not("Infinity");

trace("//~\"1.0\"");
assert_bitwise_not("1.0");

trace("//~\"-1.0\"");
assert_bitwise_not("-1.0");

trace("//~\"0xFF1306\"");
assert_bitwise_not("0xFF1306");