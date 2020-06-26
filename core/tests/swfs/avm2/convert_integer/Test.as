package {
	public class Test {
	}
}

function assert_integer(val) {
	var num_val : int = val;
	
	trace(num_val);
}

trace("//true");
assert_integer(true);

trace("//false");
assert_integer(false);

trace("//null");
assert_integer(null);

trace("//undefined");
assert_integer(undefined);

trace("//\"\"");
assert_integer("");

trace("//\"str\"");
assert_integer("str");

trace("//\"true\"");
assert_integer("true");

trace("//\"false\"");
assert_integer("false");

trace("//0.0");
assert_integer(0.0);

trace("//NaN");
assert_integer(NaN);

trace("//-0.0");
assert_integer(-0.0);

trace("//Infinity");
assert_integer(Infinity);

trace("//1.0");
assert_integer(1.0);

trace("//-1.0");
assert_integer(-1.0);

trace("//0xFF1306");
assert_integer(0xFF1306);

trace("//1.2315e2");
assert_integer(1.2315e2);

trace("//0x7FFFFFFF");
assert_integer(0x7FFFFFFF);

trace("//0x80000000");
assert_integer(0x80000000);

trace("//0x80000001");
assert_integer(0x80000001);

trace("//0x180000001");
assert_integer(0x180000001);

trace("//0x100000001");
assert_integer(0x100000001);

trace("//-0x7FFFFFFF");
assert_integer(-0x7FFFFFFF);

trace("//-0x80000000");
assert_integer(-0x80000000);

trace("//-0x80000001");
assert_integer(-0x80000001);

trace("//-0x180000001");
assert_integer(-0x180000001);

trace("//-0x100000001");
assert_integer(-0x100000001);

trace("//new Object()");
assert_integer({});

trace("//\"0.0\"");
assert_integer("0.0");

trace("//\"NaN\"");
assert_integer("NaN");

trace("//\"-0.0\"");
assert_integer("-0.0");

trace("//\"Infinity\"");
assert_integer("Infinity");

trace("//\"1.0\"");
assert_integer("1.0");

trace("//\"-1.0\"");
assert_integer("-1.0");

trace("//\"0xFF1306\"");
assert_integer("0xFF1306");

trace("//\"1.2315e2\"");
assert_integer("1.2315e2");

trace("//\"0x7FFFFFFF\"");
assert_integer(0x7FFFFFFF);

trace("//\"0x80000000\"");
assert_integer(0x80000000);

trace("//\"0x80000001\"");
assert_integer(0x80000001);

trace("//\"0x180000001\"");
assert_integer(0x180000001);

trace("//\"0x100000001\"");
assert_integer(0x100000001);

trace("//\"-0x7FFFFFFF\"");
assert_integer(-0x7FFFFFFF);

trace("//\"-0x80000000\"");
assert_integer(-0x80000000);

trace("//\"-0x80000001\"");
assert_integer(-0x80000001);

trace("//\"-0x180000001\"");
assert_integer(-0x180000001);

trace("//\"-0x100000001\"");
assert_integer(-0x100000001);