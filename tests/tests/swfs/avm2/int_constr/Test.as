package {
	public class Test {
	}
}

trace("//new int()");
trace(new int());

trace("//true");
trace(new int(true));

trace("//false");
trace(new int(false));

trace("//null");
trace(new int(null));

trace("//undefined");
trace(new int(undefined));

trace("//\"\"");
trace(new int(""));

trace("//\"str\"");
trace(new int("str"));

trace("//\"true\"");
trace(new int("true"));

trace("//\"false\"");
trace(new int("false"));

trace("//0.0");
trace(new int(0.0));

trace("//NaN");
trace(new int(NaN));

trace("//-0.0");
trace(new int(-0.0));

trace("//Infinity");
trace(new int(Infinity));

trace("//1.0");
trace(new int(1.0));

trace("//-1.0");
trace(new int(-1.0));

trace("//0xFF1306");
trace(new int(0xFF1306));

trace("//1.2315e2");
trace(new int(1.2315e2));

trace("//0x7FFFFFFF");
trace(new int(0x7FFFFFFF));

trace("//0x80000000");
trace(new int(0x80000000));

trace("//0x80000001");
trace(new int(0x80000001));

trace("//0x180000001");
trace(new int(0x180000001));

trace("//0x100000001");
trace(new int(0x100000001));

trace("//-0x7FFFFFFF");
trace(new int(-0x7FFFFFFF));

trace("//-0x80000000");
trace(new int(-0x80000000));

trace("//-0x80000001");
trace(new int(-0x80000001));

trace("//-0x180000001");
trace(new int(-0x180000001));

trace("//-0x100000001");
trace(new int(-0x100000001));

trace("//new Object()");
trace(new int({}));

trace("//\"0.0\"");
trace(new int("0.0"));

trace("//\"NaN\"");
trace(new int("NaN"));

trace("//\"-0.0\"");
trace(new int("-0.0"));

trace("//\"Infinity\"");
trace(new int("Infinity"));

trace("//\"1.0\"");
trace(new int("1.0"));

trace("//\"-1.0\"");
trace(new int("-1.0"));

trace("//\"0xFF1306\"");
trace(new int("0xFF1306"));

trace("//\"1.2315e2\"");
trace(new int("1.2315e2"));

trace("//\"0x7FFFFFFF\"");
trace(new int(0x7FFFFFFF));

trace("//\"0x80000000\"");
trace(new int(0x80000000));

trace("//\"0x80000001\"");
trace(new int(0x80000001));

trace("//\"0x180000001\"");
trace(new int(0x180000001));

trace("//\"0x100000001\"");
trace(new int(0x100000001));

trace("//\"-0x7FFFFFFF\"");
trace(new int(-0x7FFFFFFF));

trace("//\"-0x80000000\"");
trace(new int(-0x80000000));

trace("//\"-0x80000001\"");
trace(new int(-0x80000001));

trace("//\"-0x180000001\"");
trace(new int(-0x180000001));

trace("//\"-0x100000001\"");
trace(new int(-0x100000001));