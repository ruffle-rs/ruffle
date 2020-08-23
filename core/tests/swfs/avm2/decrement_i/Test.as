package {
	public class Test {
	}
}

function assert_decrement_i(val1) {
	var int1 : int = val1;
	trace(int1--);
}

trace("//true");
assert_decrement_i(true);

trace("//false");
assert_decrement_i(false);

trace("//null");
assert_decrement_i(null);

trace("//undefined");
assert_decrement_i(undefined);

trace("//\"\"");
assert_decrement_i("");

trace("//\"str\"");
assert_decrement_i("str");

trace("//\"true\"");
assert_decrement_i("true");

trace("//\"false\"");
assert_decrement_i("false");

trace("//0.0");
assert_decrement_i(0.0);

trace("//NaN");
assert_decrement_i(NaN);

trace("//-0.0");
assert_decrement_i(-0.0);

trace("//Infinity");
assert_decrement_i(Infinity);

trace("//1.0");
assert_decrement_i(1.0);

trace("//-1.0");
assert_decrement_i(-1.0);

trace("//0xFF1306");
assert_decrement_i(0xFF1306);

trace("//new Object()");
assert_decrement_i({});

trace("//\"0.0\"");
assert_decrement_i("0.0");

trace("//\"NaN\"");
assert_decrement_i("NaN");

trace("//\"-0.0\"");
assert_decrement_i("-0.0");

trace("//\"Infinity\"");
assert_decrement_i("Infinity");

trace("//\"1.0\"");
assert_decrement_i("1.0");

trace("//\"-1.0\"");
assert_decrement_i("-1.0");

trace("//\"0xFF1306\"");
assert_decrement_i("0xFF1306");