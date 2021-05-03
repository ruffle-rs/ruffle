package {
	public class Test {
	}
}

function assert_bool_not(val) {
	var notval = !val;
	
	if (notval === true) {
		trace("Inverse is true");
	} else if (notval === false) {
		trace("Inverse is false");
	} else {
		trace("TEST FAIL: Inverse is nonboolean");
	}
}

trace("//!true");
assert_bool_not(true);

trace("//!false");
assert_bool_not(false);

trace("//!null");
assert_bool_not(null);

trace("//!undefined");
assert_bool_not(undefined);

trace("//!\"\"");
assert_bool_not("");

trace("//!\"str\"");
assert_bool_not("str");

trace("//!\"true\"");
assert_bool_not("true");

trace("//!\"false\"");
assert_bool_not("false");

trace("//!0.0");
assert_bool_not(0.0);

trace("//!NaN");
assert_bool_not(NaN);

trace("//!-0.0");
assert_bool_not(-0.0);

trace("//!Infinity");
assert_bool_not(Infinity);

trace("//!1.0");
assert_bool_not(1.0);

trace("//!-1.0");
assert_bool_not(-1.0);

trace("//!new Object()");
assert_bool_not({});