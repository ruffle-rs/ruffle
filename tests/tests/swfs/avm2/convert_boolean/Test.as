package {
	public class Test {
	}
}

function assert_truthiness(val) {
	var other_val : Boolean = val;
	
	if (other_val === true) {
		trace("Value is truthy");
	} else if (other_val === false) {
		trace("Value is falsy");
	} else {
		trace("TEST FAIL: Value not converted to boolean");
	}
}

trace("//if (true)");
assert_truthiness(true);

trace("//if (false)");
assert_truthiness(false);

trace("//if (null)");
assert_truthiness(null);

trace("//if (undefined)");
assert_truthiness(undefined);

trace("//if (\"\")");
assert_truthiness("");

trace("//if (\"str\")");
assert_truthiness("str");

trace("//if (\"true\")");
assert_truthiness("true");

trace("//if (\"false\")");
assert_truthiness("false");

trace("//if (0.0)");
assert_truthiness(0.0);

trace("//if (NaN)");
assert_truthiness(NaN);

trace("//if (-0.0)");
assert_truthiness(-0.0);

trace("//if (Infinity)");
assert_truthiness(Infinity);

trace("//if (1.0)");
assert_truthiness(1.0);

trace("//if (-1.0)");
assert_truthiness(-1.0);

trace("//if (new Object())");
assert_truthiness({});