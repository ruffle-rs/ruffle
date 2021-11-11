package {
	public class Test {
	}
}

trace("//new Boolean()");
trace(new Boolean());

trace("//new Boolean(true)");
trace(new Boolean(true));

trace("//new Boolean(false)");
trace(new Boolean(false));

trace("//new Boolean(null)");
trace(new Boolean(null));

trace("//new Boolean(undefined)");
trace(new Boolean(undefined));

trace("//new Boolean(\"\")");
trace(new Boolean(""));

trace("//new Boolean(\"str\")");
trace(new Boolean("str"));

trace("//new Boolean(\"true\")");
trace(new Boolean("true"));

trace("//new Boolean(\"false\")");
trace(new Boolean("false"));

trace("//new Boolean(0.0)");
trace(new Boolean(0.0));

trace("//new Boolean(NaN)");
trace(new Boolean(NaN));

trace("//new Boolean(-0.0)");
trace(new Boolean(-0.0));

trace("//new Boolean(Infinity)");
trace(new Boolean(Infinity));

trace("//new Boolean(1.0)");
trace(new Boolean(1.0));

trace("//new Boolean(-1.0)");
trace(new Boolean(-1.0));

trace("//new Boolean({})");
trace(new Boolean({}));