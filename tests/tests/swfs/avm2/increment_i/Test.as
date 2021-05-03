package {
	public class Test {
	}
}

function assert_increment_i(val1) {
	var int1 : int = val1;
	trace(int1++);
}

trace("//true");
assert_increment_i(true);

trace("//false");
assert_increment_i(false);

trace("//null");
assert_increment_i(null);

trace("//undefined");
assert_increment_i(undefined);

trace("//\"\"");
assert_increment_i("");

trace("//\"str\"");
assert_increment_i("str");

trace("//\"true\"");
assert_increment_i("true");

trace("//\"false\"");
assert_increment_i("false");

trace("//0.0");
assert_increment_i(0.0);

trace("//NaN");
assert_increment_i(NaN);

trace("//-0.0");
assert_increment_i(-0.0);

trace("//Infinity");
assert_increment_i(Infinity);

trace("//1.0");
assert_increment_i(1.0);

trace("//-1.0");
assert_increment_i(-1.0);

trace("//0xFF1306");
assert_increment_i(0xFF1306);

trace("//new Object()");
assert_increment_i({});

trace("//\"0.0\"");
assert_increment_i("0.0");

trace("//\"NaN\"");
assert_increment_i("NaN");

trace("//\"-0.0\"");
assert_increment_i("-0.0");

trace("//\"Infinity\"");
assert_increment_i("Infinity");

trace("//\"1.0\"");
assert_increment_i("1.0");

trace("//\"-1.0\"");
assert_increment_i("-1.0");

trace("//\"0xFF1306\"");
assert_increment_i("0xFF1306");