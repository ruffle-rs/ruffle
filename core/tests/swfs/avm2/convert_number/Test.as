package {
	public class Test {
	}
}

function assert_number(val) {
	var num_val : Number = val;
	
	trace(num_val);
}

trace("//true");
assert_number(true);

trace("//false");
assert_number(false);

trace("//null");
assert_number(null);

trace("//undefined");
assert_number(undefined);

trace("//\"\"");
assert_number("");

trace("//\"str\"");
assert_number("str");

trace("//\"true\"");
assert_number("true");

trace("//\"false\"");
assert_number("false");

trace("//0.0");
assert_number(0.0);

trace("//NaN");
assert_number(NaN);

trace("//-0.0");
assert_number(-0.0);

trace("//Infinity");
assert_number(Infinity);

trace("//1.0");
assert_number(1.0);

trace("//-1.0");
assert_number(-1.0);

trace("//0xFF1306");
assert_number(0xFF1306);

trace("//1.2315e2");
assert_number(1.2315e2);

trace("//new Object()");
assert_number({});

trace("//\"0.0\"");
assert_number("0.0");

trace("//\"NaN\"");
assert_number("NaN");

trace("//\"-0.0\"");
assert_number("-0.0");

trace("//\"Infinity\"");
assert_number("Infinity");

trace("//\"-Infinity\"");
assert_number("-Infinity");

trace("//\"infinity\"");
assert_number("infinity");

trace("//\"inf\"");
assert_number("inf");

trace("//\"1.0\"");
assert_number("1.0");

trace("//\"-1.0\"");
assert_number("-1.0");

trace("//\"0xFF1306\"");
assert_number("0xFF1306");

trace("//\"1.2315e2\"");
assert_number("1.2315e2");