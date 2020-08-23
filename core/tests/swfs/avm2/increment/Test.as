package {
	public class Test {
	}
}

function assert_increment(val1) {
	trace(val1++);
}

trace("//true");
assert_increment(true);

trace("//false");
assert_increment(false);

trace("//null");
assert_increment(null);

trace("//undefined");
assert_increment(undefined);

trace("//\"\"");
assert_increment("");

trace("//\"str\"");
assert_increment("str");

trace("//\"true\"");
assert_increment("true");

trace("//\"false\"");
assert_increment("false");

trace("//0.0");
assert_increment(0.0);

trace("//NaN");
assert_increment(NaN);

trace("//-0.0");
assert_increment(-0.0);

trace("//Infinity");
assert_increment(Infinity);

trace("//1.0");
assert_increment(1.0);

trace("//-1.0");
assert_increment(-1.0);

trace("//0xFF1306");
assert_increment(0xFF1306);

trace("//new Object()");
assert_increment({});

trace("//\"0.0\"");
assert_increment("0.0");

trace("//\"NaN\"");
assert_increment("NaN");

trace("//\"-0.0\"");
assert_increment("-0.0");

trace("//\"Infinity\"");
assert_increment("Infinity");

trace("//\"1.0\"");
assert_increment("1.0");

trace("//\"-1.0\"");
assert_increment("-1.0");

trace("//\"0xFF1306\"");
assert_increment("0xFF1306");