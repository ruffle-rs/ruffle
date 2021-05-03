package {
	public class Test {
	}
}

function assert_decrement(val1) {
	trace(val1--);
}

trace("//true");
assert_decrement(true);

trace("//false");
assert_decrement(false);

trace("//null");
assert_decrement(null);

trace("//undefined");
assert_decrement(undefined);

trace("//\"\"");
assert_decrement("");

trace("//\"str\"");
assert_decrement("str");

trace("//\"true\"");
assert_decrement("true");

trace("//\"false\"");
assert_decrement("false");

trace("//0.0");
assert_decrement(0.0);

trace("//NaN");
assert_decrement(NaN);

trace("//-0.0");
assert_decrement(-0.0);

trace("//Infinity");
assert_decrement(Infinity);

trace("//1.0");
assert_decrement(1.0);

trace("//-1.0");
assert_decrement(-1.0);

trace("//0xFF1306");
assert_decrement(0xFF1306);

trace("//new Object()");
assert_decrement({});

trace("//\"0.0\"");
assert_decrement("0.0");

trace("//\"NaN\"");
assert_decrement("NaN");

trace("//\"-0.0\"");
assert_decrement("-0.0");

trace("//\"Infinity\"");
assert_decrement("Infinity");

trace("//\"1.0\"");
assert_decrement("1.0");

trace("//\"-1.0\"");
assert_decrement("-1.0");

trace("//\"0xFF1306\"");
assert_decrement("0xFF1306");