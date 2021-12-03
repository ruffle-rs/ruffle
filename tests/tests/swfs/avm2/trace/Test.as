package {
	public class Test {}
}

// Test tracing with primitive parameters.

trace("test");
trace("trace", "with", "multiple", "params");
trace(false, true, undefined, null, 0, 1, 1.2345, "string", {});
trace();

// Test that \r is normalized to \n.

trace('// "a\\rb"');
trace("a\rb");
trace();

trace('// "a\\r\\nb"');
trace("a\r\nb");
trace();
