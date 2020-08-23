package {
	public class Test {
	}
}

function assert_negate(val) {
	trace(-val);
}

trace("//-true");
assert_negate(true);

trace("//-false");
assert_negate(false);

trace("//-null");
assert_negate(null);

trace("//-undefined");
assert_negate(undefined);

trace("//-\"\"");
assert_negate("");

trace("//-\"str\"");
assert_negate("str");

trace("//-\"true\"");
assert_negate("true");

trace("//-\"false\"");
assert_negate("false");

trace("//-0.0");
assert_negate(0.0);

trace("//-NaN");
assert_negate(NaN);

trace("//--0.0");
assert_negate(-0.0);

trace("//-Infinity");
assert_negate(Infinity);

trace("//-1.0");
assert_negate(1.0);

trace("//--1.0");
assert_negate(-1.0);

trace("//-new Object()");
assert_negate({});