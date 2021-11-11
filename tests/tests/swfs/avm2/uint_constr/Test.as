package {
	public class Test {
	}
}

trace("//new uint()");
trace(new uint());

trace("//true");
trace(new uint(true));

trace("//false");
trace(new uint(false));

trace("//null");
trace(new uint(null));

trace("//undefined");
trace(new uint(undefined));

trace("//\"\"");
trace(new uint(""));

trace("//\"str\"");
trace(new uint("str"));

trace("//\"true\"");
trace(new uint("true"));

trace("//\"false\"");
trace(new uint("false"));

trace("//0.0");
trace(new uint(0.0));

trace("//NaN");
trace(new uint(NaN));

trace("//-0.0");
trace(new uint(-0.0));

trace("//Infinity");
trace(new uint(Infinity));

trace("//1.0");
trace(new uint(1.0));

trace("//-1.0");
trace(new uint(-1.0));

trace("//0xFF1306");
trace(new uint(0xFF1306));

trace("//1.2315e2");
trace(new uint(1.2315e2));

trace("//0x7FFFFFFF");
trace(new uint(0x7FFFFFFF));

trace("//0x80000000");
trace(new uint(0x80000000));

trace("//0x80000001");
trace(new uint(0x80000001));

trace("//0x180000001");
trace(new uint(0x180000001));

trace("//0x100000001");
trace(new uint(0x100000001));

trace("//-0x7FFFFFFF");
trace(new uint(-0x7FFFFFFF));

trace("//-0x80000000");
trace(new uint(-0x80000000));

trace("//-0x80000001");
trace(new uint(-0x80000001));

trace("//-0x180000001");
trace(new uint(-0x180000001));

trace("//-0x100000001");
trace(new uint(-0x100000001));

trace("//new Object()");
trace(new uint({}));

trace("//\"0.0\"");
trace(new uint("0.0"));

trace("//\"NaN\"");
trace(new uint("NaN"));

trace("//\"-0.0\"");
trace(new uint("-0.0"));

trace("//\"Infinity\"");
trace(new uint("Infinity"));

trace("//\"1.0\"");
trace(new uint("1.0"));

trace("//\"-1.0\"");
trace(new uint("-1.0"));

trace("//\"0xFF1306\"");
trace(new uint("0xFF1306"));

trace("//\"1.2315e2\"");
trace(new uint("1.2315e2"));

trace("//\"0x7FFFFFFF\"");
trace(new uint(0x7FFFFFFF));

trace("//\"0x80000000\"");
trace(new uint(0x80000000));

trace("//\"0x80000001\"");
trace(new uint(0x80000001));

trace("//\"0x180000001\"");
trace(new uint(0x180000001));

trace("//\"0x100000001\"");
trace(new uint(0x100000001));

trace("//\"-0x7FFFFFFF\"");
trace(new uint(-0x7FFFFFFF));

trace("//\"-0x80000000\"");
trace(new uint(-0x80000000));

trace("//\"-0x80000001\"");
trace(new uint(-0x80000001));

trace("//\"-0x180000001\"");
trace(new uint(-0x180000001));

trace("//\"-0x100000001\"");
trace(new uint(-0x100000001));