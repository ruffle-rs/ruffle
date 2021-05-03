package {
	public class Test {}
}

// Test tracing with primitive parameters.
trace();
trace("test");
trace("trace", "with", "multiple", "params");
trace(false, true, undefined, null, 0, 1, 1.2345, "string", {});
