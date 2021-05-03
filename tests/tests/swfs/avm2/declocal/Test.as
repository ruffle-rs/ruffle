package {
	public class Test {
	}
}

function assert_declocal(val1) {
	val1--;
	trace(val1);
}

trace("//true");
assert_declocal(true);

trace("//false");
assert_declocal(false);

trace("//null");
assert_declocal(null);

trace("//undefined");
assert_declocal(undefined);

trace("//\"\"");
assert_declocal("");

trace("//\"str\"");
assert_declocal("str");

trace("//\"true\"");
assert_declocal("true");

trace("//\"false\"");
assert_declocal("false");

trace("//0.0");
assert_declocal(0.0);

trace("//NaN");
assert_declocal(NaN);

trace("//-0.0");
assert_declocal(-0.0);

trace("//Infinity");
assert_declocal(Infinity);

trace("//1.0");
assert_declocal(1.0);

trace("//-1.0");
assert_declocal(-1.0);

trace("//0xFF1306");
assert_declocal(0xFF1306);

trace("//new Object()");
assert_declocal({});

trace("//\"0.0\"");
assert_declocal("0.0");

trace("//\"NaN\"");
assert_declocal("NaN");

trace("//\"-0.0\"");
assert_declocal("-0.0");

trace("//\"Infinity\"");
assert_declocal("Infinity");

trace("//\"1.0\"");
assert_declocal("1.0");

trace("//\"-1.0\"");
assert_declocal("-1.0");

trace("//\"0xFF1306\"");
assert_declocal("0xFF1306");