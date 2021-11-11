package {
	public class Test {
	}
}

trace("//new Number()");
trace(new Number());

trace("//true");
trace(new Number(true));

trace("//false");
trace(new Number(false));

trace("//null");
trace(new Number(null));

trace("//undefined");
trace(new Number(undefined));

trace("//\"\"");
trace(new Number(""));

trace("//\"str\"");
trace(new Number("str"));

trace("//\"true\"");
trace(new Number("true"));

trace("//\"false\"");
trace(new Number("false"));

trace("//0.0");
trace(new Number(0.0));

trace("//NaN");
trace(new Number(NaN));

trace("//-0.0");
trace(new Number(-0.0));

trace("//Infinity");
trace(new Number(Infinity));

trace("//1.0");
trace(new Number(1.0));

trace("//-1.0");
trace(new Number(-1.0));

trace("//0xFF1306");
trace(new Number(0xFF1306));

trace("//1.2315e2");
trace(new Number(1.2315e2));

trace("//new Object()");
trace(new Number({}));

trace("//\"0.0\"");
trace(new Number("0.0"));

trace("//\"NaN\"");
trace(new Number("NaN"));

trace("//\"-0.0\"");
trace(new Number("-0.0"));

trace("//\"Infinity\"");
trace(new Number("Infinity"));

trace("//\"-Infinity\"");
trace(new Number("-Infinity"));

trace("//\"infinity\"");
trace(new Number("infinity"));

trace("//\"inf\"");
trace(new Number("inf"));

trace("//\"1.0\"");
trace(new Number("1.0"));

trace("//\"-1.0\"");
trace(new Number("-1.0"));

trace("//\"0xFF1306\"");
trace(new Number("0xFF1306"));

trace("//\"1.2315e2\"");
trace(new Number("1.2315e2"));