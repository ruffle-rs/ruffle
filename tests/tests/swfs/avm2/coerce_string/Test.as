package {
	public class Test {
	}
}

function assert_string(val) {
	var num_val : String = val;
	
	trace(num_val);
}

trace("//true");
assert_string(true);

trace("//false");
assert_string(false);

trace("//null");
assert_string(null);

trace("//undefined");
assert_string(undefined);

trace("//\"\"");
assert_string("");

trace("//\"str\"");
assert_string("str");

trace("//\"true\"");
assert_string("true");

trace("//\"false\"");
assert_string("false");

trace("//0.0");
assert_string(0.0);

trace("//NaN");
assert_string(NaN);

trace("//-0.0");
assert_string(-0.0);

trace("//Infinity");
assert_string(Infinity);

trace("//1.0");
assert_string(1.0);

trace("//-1.0");
assert_string(-1.0);

trace("//0xFF1306");
assert_string(0xFF1306);

trace("//0x7FFFFFFF");
assert_string(0x7FFFFFFF);

trace("//0x80000000");
assert_string(0x80000000);

trace("//0x80000001");
assert_string(0x80000001);

trace("//0x180000001");
assert_string(0x180000001);

trace("//0x100000001");
assert_string(0x100000001);

trace("//-0x7FFFFFFF");
assert_string(-0x7FFFFFFF);

trace("//-0x80000000");
assert_string(-0x80000000);

trace("//-0x80000001");
assert_string(-0x80000001);

trace("//-0x180000001");
assert_string(-0x180000001);

trace("//-0x100000001");
assert_string(-0x100000001);

trace("//new Object()");
assert_string({});

trace("//\"0.0\"");
assert_string("0.0");

trace("//\"NaN\"");
assert_string("NaN");

trace("//\"-0.0\"");
assert_string("-0.0");

trace("//\"Infinity\"");
assert_string("Infinity");

trace("//\"1.0\"");
assert_string("1.0");

trace("//\"-1.0\"");
assert_string("-1.0");

trace("//\"0xFF1306\"");
assert_string("0xFF1306");

trace("//\"0x7FFFFFFF\"");
assert_string(0x7FFFFFFF);

trace("//\"0x80000000\"");
assert_string(0x80000000);

trace("//\"0x80000001\"");
assert_string(0x80000001);

trace("//\"0x180000001\"");
assert_string(0x180000001);

trace("//\"0x100000001\"");
assert_string(0x100000001);

trace("//\"-0x7FFFFFFF\"");
assert_string(-0x7FFFFFFF);

trace("//\"-0x80000000\"");
assert_string(-0x80000000);

trace("//\"-0x80000001\"");
assert_string(-0x80000001);

trace("//\"-0x180000001\"");
assert_string(-0x180000001);

trace("//\"-0x100000001\"");
assert_string(-0x100000001);