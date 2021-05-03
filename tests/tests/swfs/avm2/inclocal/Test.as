package {
	public class Test {
	}
}

function assert_inclocal(val1) {
	val1++;
	trace(val1);
}

trace("//true");
assert_inclocal(true);

trace("//false");
assert_inclocal(false);

trace("//null");
assert_inclocal(null);

trace("//undefined");
assert_inclocal(undefined);

trace("//\"\"");
assert_inclocal("");

trace("//\"str\"");
assert_inclocal("str");

trace("//\"true\"");
assert_inclocal("true");

trace("//\"false\"");
assert_inclocal("false");

trace("//0.0");
assert_inclocal(0.0);

trace("//NaN");
assert_inclocal(NaN);

trace("//-0.0");
assert_inclocal(-0.0);

trace("//Infinity");
assert_inclocal(Infinity);

trace("//1.0");
assert_inclocal(1.0);

trace("//-1.0");
assert_inclocal(-1.0);

trace("//0xFF1306");
assert_inclocal(0xFF1306);

trace("//new Object()");
assert_inclocal({});

trace("//\"0.0\"");
assert_inclocal("0.0");

trace("//\"NaN\"");
assert_inclocal("NaN");

trace("//\"-0.0\"");
assert_inclocal("-0.0");

trace("//\"Infinity\"");
assert_inclocal("Infinity");

trace("//\"1.0\"");
assert_inclocal("1.0");

trace("//\"-1.0\"");
assert_inclocal("-1.0");

trace("//\"0xFF1306\"");
assert_inclocal("0xFF1306");