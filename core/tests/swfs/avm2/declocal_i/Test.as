package {
	public class Test {
	}
}

function assert_declocal_i(val1) {
	var int1 : int = val1;
	int1--;
	trace(int1);
}

trace("//true");
assert_declocal_i(true);

trace("//false");
assert_declocal_i(false);

trace("//null");
assert_declocal_i(null);

trace("//undefined");
assert_declocal_i(undefined);

trace("//\"\"");
assert_declocal_i("");

trace("//\"str\"");
assert_declocal_i("str");

trace("//\"true\"");
assert_declocal_i("true");

trace("//\"false\"");
assert_declocal_i("false");

trace("//0.0");
assert_declocal_i(0.0);

trace("//NaN");
assert_declocal_i(NaN);

trace("//-0.0");
assert_declocal_i(-0.0);

trace("//Infinity");
assert_declocal_i(Infinity);

trace("//1.0");
assert_declocal_i(1.0);

trace("//-1.0");
assert_declocal_i(-1.0);

trace("//0xFF1306");
assert_declocal_i(0xFF1306);

trace("//new Object()");
assert_declocal_i({});

trace("//\"0.0\"");
assert_declocal_i("0.0");

trace("//\"NaN\"");
assert_declocal_i("NaN");

trace("//\"-0.0\"");
assert_declocal_i("-0.0");

trace("//\"Infinity\"");
assert_declocal_i("Infinity");

trace("//\"1.0\"");
assert_declocal_i("1.0");

trace("//\"-1.0\"");
assert_declocal_i("-1.0");

trace("//\"0xFF1306\"");
assert_declocal_i("0xFF1306");