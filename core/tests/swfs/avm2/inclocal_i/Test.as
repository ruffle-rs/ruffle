package {
	public class Test {
	}
}

function assert_inclocal_i(val1) {
	var int1 : int = val1;
	int1++;
	trace(int1);
}

trace("//true");
assert_inclocal_i(true);

trace("//false");
assert_inclocal_i(false);

trace("//null");
assert_inclocal_i(null);

trace("//undefined");
assert_inclocal_i(undefined);

trace("//\"\"");
assert_inclocal_i("");

trace("//\"str\"");
assert_inclocal_i("str");

trace("//\"true\"");
assert_inclocal_i("true");

trace("//\"false\"");
assert_inclocal_i("false");

trace("//0.0");
assert_inclocal_i(0.0);

trace("//NaN");
assert_inclocal_i(NaN);

trace("//-0.0");
assert_inclocal_i(-0.0);

trace("//Infinity");
assert_inclocal_i(Infinity);

trace("//1.0");
assert_inclocal_i(1.0);

trace("//-1.0");
assert_inclocal_i(-1.0);

trace("//0xFF1306");
assert_inclocal_i(0xFF1306);

trace("//new Object()");
assert_inclocal_i({});

trace("//\"0.0\"");
assert_inclocal_i("0.0");

trace("//\"NaN\"");
assert_inclocal_i("NaN");

trace("//\"-0.0\"");
assert_inclocal_i("-0.0");

trace("//\"Infinity\"");
assert_inclocal_i("Infinity");

trace("//\"1.0\"");
assert_inclocal_i("1.0");

trace("//\"-1.0\"");
assert_inclocal_i("-1.0");

trace("//\"0xFF1306\"");
assert_inclocal_i("0xFF1306");