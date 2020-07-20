package {
	public class Test {
	}
}

class TestWithVars {
	public var prop;
	public var propDefault = "y.propDefault resolved!";
	public const propConst = "y.propConst resolved!";
}

class ExtendedTest extends TestWithVars {
	public var prop2;
	public var prop2Default = "z.prop2Default resolved!";
	public const prop2Const = "z.prop2Const resolved!";
}

var x = {};

x.prop = "x.prop resolved!";
trace(x.prop);

var y = new TestWithVars();

y.prop = "y.prop resolved!";
trace(y.prop);

trace(y.propDefault);
y.propDefault = "y.propDefault overwritten!";
trace(y.propDefault);

trace(y.propConst);

var z = new ExtendedTest();

z.prop = "z.prop resolved!";
trace(z.prop);

z.prop2 = "z.prop2 resolved!";
trace(z.prop2);

trace(z.propDefault);
trace(z.prop2Default);
z.propDefault = "TEST FAIL: Default overrides should not affect other instances!";
trace(y.propDefault);

trace(z.propConst);