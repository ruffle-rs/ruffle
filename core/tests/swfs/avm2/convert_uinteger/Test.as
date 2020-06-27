package {
	public class Test {
	}
}

function assert_uinteger(val) {
	var num_val : uint = val;
	
	trace(num_val);
}

trace("//true");
assert_uinteger(true);

trace("//false");
assert_uinteger(false);

trace("//null");
assert_uinteger(null);

trace("//undefined");
assert_uinteger(undefined);

trace("//\"\"");
assert_uinteger("");

trace("//\"str\"");
assert_uinteger("str");

trace("//\"true\"");
assert_uinteger("true");

trace("//\"false\"");
assert_uinteger("false");

trace("//0.0");
assert_uinteger(0.0);

trace("//NaN");
assert_uinteger(NaN);

trace("//-0.0");
assert_uinteger(-0.0);

trace("//Infinity");
assert_uinteger(Infinity);

trace("//1.0");
assert_uinteger(1.0);

trace("//-1.0");
assert_uinteger(-1.0);

trace("//0xFF1306");
assert_uinteger(0xFF1306);

trace("//1.2315e2");
assert_uinteger(1.2315e2);

trace("//0x7FFFFFFF");
assert_uinteger(0x7FFFFFFF);

trace("//0x80000000");
assert_uinteger(0x80000000);

trace("//0x80000001");
assert_uinteger(0x80000001);

trace("//0x180000001");
assert_uinteger(0x180000001);

trace("//0x100000001");
assert_uinteger(0x100000001);

trace("//-0x7FFFFFFF");
assert_uinteger(-0x7FFFFFFF);

trace("//-0x80000000");
assert_uinteger(-0x80000000);

trace("//-0x80000001");
assert_uinteger(-0x80000001);

trace("//-0x180000001");
assert_uinteger(-0x180000001);

trace("//-0x100000001");
assert_uinteger(-0x100000001);

trace("//new Object()");
assert_uinteger({});

trace("//\"0.0\"");
assert_uinteger("0.0");

trace("//\"NaN\"");
assert_uinteger("NaN");

trace("//\"-0.0\"");
assert_uinteger("-0.0");

trace("//\"Infinity\"");
assert_uinteger("Infinity");

trace("//\"1.0\"");
assert_uinteger("1.0");

trace("//\"-1.0\"");
assert_uinteger("-1.0");

trace("//\"0xFF1306\"");
assert_uinteger("0xFF1306");

trace("//\"1.2315e2\"");
assert_uinteger("1.2315e2");

trace("//\"0x7FFFFFFF\"");
assert_uinteger(0x7FFFFFFF);

trace("//\"0x80000000\"");
assert_uinteger(0x80000000);

trace("//\"0x80000001\"");
assert_uinteger(0x80000001);

trace("//\"0x180000001\"");
assert_uinteger(0x180000001);

trace("//\"0x100000001\"");
assert_uinteger(0x100000001);

trace("//\"-0x7FFFFFFF\"");
assert_uinteger(-0x7FFFFFFF);

trace("//\"-0x80000000\"");
assert_uinteger(-0x80000000);

trace("//\"-0x80000001\"");
assert_uinteger(-0x80000001);

trace("//\"-0x180000001\"");
assert_uinteger(-0x180000001);

trace("//\"-0x100000001\"");
assert_uinteger(-0x100000001);