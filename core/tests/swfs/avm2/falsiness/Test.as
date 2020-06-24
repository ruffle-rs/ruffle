package {
	public class Test {
	}
}

function assert_falsiness(val) {
	if (!val) {
		trace("Value is falsy");
	} else {
		trace("Value is truthy");
	}
}

trace("//if (!true)");
assert_falsiness(true);

trace("//if (!false)");
assert_falsiness(false);

trace("//if (!null)");
assert_falsiness(null);

trace("//if (!undefined)");
assert_falsiness(undefined);

trace("//if (!\"\")");
assert_falsiness("");

trace("//if (!\"str\")");
assert_falsiness("str");

trace("//if (!\"true\")");
assert_falsiness("true");

trace("//if (!\"false\")");
assert_falsiness("false");

trace("//if (!0.0)");
assert_falsiness(0.0);

trace("//if (!NaN)");
assert_falsiness(NaN);

trace("//if (!-0.0)");
assert_falsiness(-0.0);

trace("//if (!Infinity)");
assert_falsiness(Infinity);

trace("//if (!1.0)");
assert_falsiness(1.0);

trace("//if (!-1.0)");
assert_falsiness(-1.0);

trace("//if (!new Object())");
assert_falsiness({});