package {
	public class Test {
	}
}

function assert_bitand(val1, val2) {
	trace(val1 & val2);
}

trace("//true & true");
assert_bitand(true, true);

trace("//false & true");
assert_bitand(false, true);

trace("//null & true");
assert_bitand(null, true);

trace("//undefined & true");
assert_bitand(undefined, true);

trace("//\"\" & true");
assert_bitand("", true);

trace("//\"str\" & true");
assert_bitand("str", true);

trace("//\"true\" & true");
assert_bitand("true", true);

trace("//\"false\" & true");
assert_bitand("false", true);

trace("//0.0 & true");
assert_bitand(0.0, true);

trace("//NaN & true");
assert_bitand(NaN, true);

trace("//-0.0 & true");
assert_bitand(-0.0, true);

trace("//Infinity & true");
assert_bitand(Infinity, true);

trace("//1.0 & true");
assert_bitand(1.0, true);

trace("//-1.0 & true");
assert_bitand(-1.0, true);

trace("//0xFF1306 & true");
assert_bitand(0xFF1306, true);

trace("//new Object() & true");
assert_bitand({}, true);

trace("//\"0.0\" & true");
assert_bitand("0.0", true);

trace("//\"NaN\" & true");
assert_bitand("NaN", true);

trace("//\"-0.0\" & true");
assert_bitand("-0.0", true);

trace("//\"Infinity\" & true");
assert_bitand("Infinity", true);

trace("//\"1.0\" & true");
assert_bitand("1.0", true);

trace("//\"-1.0\" & true");
assert_bitand("-1.0", true);

trace("//\"0xFF1306\" & true");
assert_bitand("0xFF1306", true);

trace("//true & false");
assert_bitand(true, false);

trace("//false & false");
assert_bitand(false, false);

trace("//null & false");
assert_bitand(null, false);

trace("//undefined & false");
assert_bitand(undefined, false);

trace("//\"\" & false");
assert_bitand("", false);

trace("//\"str\" & false");
assert_bitand("str", false);

trace("//\"true\" & false");
assert_bitand("true", false);

trace("//\"false\" & false");
assert_bitand("false", false);

trace("//0.0 & false");
assert_bitand(0.0, false);

trace("//NaN & false");
assert_bitand(NaN, false);

trace("//-0.0 & false");
assert_bitand(-0.0, false);

trace("//Infinity & false");
assert_bitand(Infinity, false);

trace("//1.0 & false");
assert_bitand(1.0, false);

trace("//-1.0 & false");
assert_bitand(-1.0, false);

trace("//0xFF1306 & false");
assert_bitand(0xFF1306, false);

trace("//new Object() & false");
assert_bitand({}, false);

trace("//\"0.0\" & false");
assert_bitand("0.0", false);

trace("//\"NaN\" & false");
assert_bitand("NaN", false);

trace("//\"-0.0\" & false");
assert_bitand("-0.0", false);

trace("//\"Infinity\" & false");
assert_bitand("Infinity", false);

trace("//\"1.0\" & false");
assert_bitand("1.0", false);

trace("//\"-1.0\" & false");
assert_bitand("-1.0", false);

trace("//\"0xFF1306\" & false");
assert_bitand("0xFF1306", false);
trace("//true & null");
assert_bitand(true, null);

trace("//false & null");
assert_bitand(false, null);

trace("//null & null");
assert_bitand(null, null);

trace("//undefined & null");
assert_bitand(undefined, null);

trace("//\"\" & null");
assert_bitand("", null);

trace("//\"str\" & null");
assert_bitand("str", null);

trace("//\"true\" & null");
assert_bitand("true", null);

trace("//\"false\" & null");
assert_bitand("false", null);

trace("//0.0 & null");
assert_bitand(0.0, null);

trace("//NaN & null");
assert_bitand(NaN, null);

trace("//-0.0 & null");
assert_bitand(-0.0, null);

trace("//Infinity & null");
assert_bitand(Infinity, null);

trace("//1.0 & null");
assert_bitand(1.0, null);

trace("//-1.0 & null");
assert_bitand(-1.0, null);

trace("//0xFF1306 & null");
assert_bitand(0xFF1306, null);

trace("//new Object() & null");
assert_bitand({}, null);

trace("//\"0.0\" & null");
assert_bitand("0.0", null);

trace("//\"NaN\" & null");
assert_bitand("NaN", null);

trace("//\"-0.0\" & null");
assert_bitand("-0.0", null);

trace("//\"Infinity\" & null");
assert_bitand("Infinity", null);

trace("//\"1.0\" & null");
assert_bitand("1.0", null);

trace("//\"-1.0\" & null");
assert_bitand("-1.0", null);

trace("//\"0xFF1306\" & null");
assert_bitand("0xFF1306", null);

trace("//true & undefined");
assert_bitand(true, undefined);

trace("//false & undefined");
assert_bitand(false, undefined);

trace("//null & undefined");
assert_bitand(null, undefined);

trace("//undefined & undefined");
assert_bitand(undefined, undefined);

trace("//\"\" & undefined");
assert_bitand("", undefined);

trace("//\"str\" & undefined");
assert_bitand("str", undefined);

trace("//\"true\" & undefined");
assert_bitand("true", undefined);

trace("//\"false\" & undefined");
assert_bitand("false", undefined);

trace("//0.0 & undefined");
assert_bitand(0.0, undefined);

trace("//NaN & undefined");
assert_bitand(NaN, undefined);

trace("//-0.0 & undefined");
assert_bitand(-0.0, undefined);

trace("//Infinity & undefined");
assert_bitand(Infinity, undefined);

trace("//1.0 & undefined");
assert_bitand(1.0, undefined);

trace("//-1.0 & undefined");
assert_bitand(-1.0, undefined);

trace("//0xFF1306 & undefined");
assert_bitand(0xFF1306, undefined);

trace("//new Object() & undefined");
assert_bitand({}, undefined);

trace("//\"0.0\" & undefined");
assert_bitand("0.0", undefined);

trace("//\"NaN\" & undefined");
assert_bitand("NaN", undefined);

trace("//\"-0.0\" & undefined");
assert_bitand("-0.0", undefined);

trace("//\"Infinity\" & undefined");
assert_bitand("Infinity", undefined);

trace("//\"1.0\" & undefined");
assert_bitand("1.0", undefined);

trace("//\"-1.0\" & undefined");
assert_bitand("-1.0", undefined);

trace("//\"0xFF1306\" & undefined");
assert_bitand("0xFF1306", undefined);

trace("//true & \"\"");
assert_bitand(true, "");

trace("//false & \"\"");
assert_bitand(false, "");

trace("//null & \"\"");
assert_bitand(null, "");

trace("//undefined & \"\"");
assert_bitand(undefined, "");

trace("//\"\" & \"\"");
assert_bitand("", "");

trace("//\"str\" & \"\"");
assert_bitand("str", "");

trace("//\"true\" & \"\"");
assert_bitand("true", "");

trace("//\"false\" & \"\"");
assert_bitand("false", "");

trace("//0.0 & \"\"");
assert_bitand(0.0, "");

trace("//NaN & \"\"");
assert_bitand(NaN, "");

trace("//-0.0 & \"\"");
assert_bitand(-0.0, "");

trace("//Infinity & \"\"");
assert_bitand(Infinity, "");

trace("//1.0 & \"\"");
assert_bitand(1.0, "");

trace("//-1.0 & \"\"");
assert_bitand(-1.0, "");

trace("//0xFF1306 & \"\"");
assert_bitand(0xFF1306, "");

trace("//new Object() & \"\"");
assert_bitand({}, "");

trace("//\"0.0\" & \"\"");
assert_bitand("0.0", "");

trace("//\"NaN\" & \"\"");
assert_bitand("NaN", "");

trace("//\"-0.0\" & \"\"");
assert_bitand("-0.0", "");

trace("//\"Infinity\" & \"\"");
assert_bitand("Infinity", "");

trace("//\"1.0\" & \"\"");
assert_bitand("1.0", "");

trace("//\"-1.0\" & \"\"");
assert_bitand("-1.0", "");

trace("//\"0xFF1306\" & \"\"");
assert_bitand("0xFF1306", "");

trace("//true & \"str\"");
assert_bitand(true, "str");

trace("//false & \"str\"");
assert_bitand(false, "str");

trace("//null & \"str\"");
assert_bitand(null, "str");

trace("//undefined & \"str\"");
assert_bitand(undefined, "str");

trace("//\"\" & \"str\"");
assert_bitand("", "str");

trace("//\"str\" & \"str\"");
assert_bitand("str", "str");

trace("//\"true\" & \"str\"");
assert_bitand("true", "str");

trace("//\"false\" & \"str\"");
assert_bitand("false", "str");

trace("//0.0 & \"str\"");
assert_bitand(0.0, "str");

trace("//NaN & \"str\"");
assert_bitand(NaN, "str");

trace("//-0.0 & \"str\"");
assert_bitand(-0.0, "str");

trace("//Infinity & \"str\"");
assert_bitand(Infinity, "str");

trace("//1.0 & \"str\"");
assert_bitand(1.0, "str");

trace("//-1.0 & \"str\"");
assert_bitand(-1.0, "str");

trace("//0xFF1306 & \"str\"");
assert_bitand(0xFF1306, "str");

trace("//new Object() & \"str\"");
assert_bitand({}, "str");

trace("//\"0.0\" & \"str\"");
assert_bitand("0.0", "str");

trace("//\"NaN\" & \"str\"");
assert_bitand("NaN", "str");

trace("//\"-0.0\" & \"str\"");
assert_bitand("-0.0", "str");

trace("//\"Infinity\" & \"str\"");
assert_bitand("Infinity", "str");

trace("//\"1.0\" & \"str\"");
assert_bitand("1.0", "str");

trace("//\"-1.0\" & \"str\"");
assert_bitand("-1.0", "str");

trace("//\"0xFF1306\" & \"str\"");
assert_bitand("0xFF1306", "str");

trace("//true & \"true\"");
assert_bitand(true, "true");

trace("//false & \"true\"");
assert_bitand(false, "true");

trace("//null & \"true\"");
assert_bitand(null, "true");

trace("//undefined & \"true\"");
assert_bitand(undefined, "true");

trace("//\"\" & \"true\"");
assert_bitand("", "true");

trace("//\"str\" & \"true\"");
assert_bitand("str", "true");

trace("//\"true\" & \"true\"");
assert_bitand("true", "true");

trace("//\"false\" & \"true\"");
assert_bitand("false", "true");

trace("//0.0 & \"true\"");
assert_bitand(0.0, "true");

trace("//NaN & \"true\"");
assert_bitand(NaN, "true");

trace("//-0.0 & \"true\"");
assert_bitand(-0.0, "true");

trace("//Infinity & \"true\"");
assert_bitand(Infinity, "true");

trace("//1.0 & \"true\"");
assert_bitand(1.0, "true");

trace("//-1.0 & \"true\"");
assert_bitand(-1.0, "true");

trace("//0xFF1306 & \"true\"");
assert_bitand(0xFF1306, "true");

trace("//new Object() & \"true\"");
assert_bitand({}, "true");

trace("//\"0.0\" & \"true\"");
assert_bitand("0.0", "true");

trace("//\"NaN\" & \"true\"");
assert_bitand("NaN", "true");

trace("//\"-0.0\" & \"true\"");
assert_bitand("-0.0", "true");

trace("//\"Infinity\" & \"true\"");
assert_bitand("Infinity", "true");

trace("//\"1.0\" & \"true\"");
assert_bitand("1.0", "true");

trace("//\"-1.0\" & \"true\"");
assert_bitand("-1.0", "true");

trace("//\"0xFF1306\" & \"true\"");
assert_bitand("0xFF1306", "true");

trace("//true & \"false\"");
assert_bitand(true, "false");

trace("//false & \"false\"");
assert_bitand(false, "false");

trace("//null & \"false\"");
assert_bitand(null, "false");

trace("//undefined & \"false\"");
assert_bitand(undefined, "false");

trace("//\"\" & \"false\"");
assert_bitand("", "false");

trace("//\"str\" & \"false\"");
assert_bitand("str", "false");

trace("//\"true\" & \"false\"");
assert_bitand("true", "false");

trace("//\"false\" & \"false\"");
assert_bitand("false", "false");

trace("//0.0 & \"false\"");
assert_bitand(0.0, "false");

trace("//NaN & \"false\"");
assert_bitand(NaN, "false");

trace("//-0.0 & \"false\"");
assert_bitand(-0.0, "false");

trace("//Infinity & \"false\"");
assert_bitand(Infinity, "false");

trace("//1.0 & \"false\"");
assert_bitand(1.0, "false");

trace("//-1.0 & \"false\"");
assert_bitand(-1.0, "false");

trace("//0xFF1306 & \"false\"");
assert_bitand(0xFF1306, "false");

trace("//new Object() & \"false\"");
assert_bitand({}, "false");

trace("//\"0.0\" & \"false\"");
assert_bitand("0.0", "false");

trace("//\"NaN\" & \"false\"");
assert_bitand("NaN", "false");

trace("//\"-0.0\" & \"false\"");
assert_bitand("-0.0", "false");

trace("//\"Infinity\" & \"false\"");
assert_bitand("Infinity", "false");

trace("//\"1.0\" & \"false\"");
assert_bitand("1.0", "false");

trace("//\"-1.0\" & \"false\"");
assert_bitand("-1.0", "false");

trace("//\"0xFF1306\" & \"false\"");
assert_bitand("0xFF1306", "false");

trace("//true & 0.0");
assert_bitand(true, 0.0);

trace("//false & 0.0");
assert_bitand(false, 0.0);

trace("//null & 0.0");
assert_bitand(null, 0.0);

trace("//undefined & 0.0");
assert_bitand(undefined, 0.0);

trace("//\"\" & 0.0");
assert_bitand("", 0.0);

trace("//\"str\" & 0.0");
assert_bitand("str", 0.0);

trace("//\"true\" & 0.0");
assert_bitand("true", 0.0);

trace("//\"false\" & 0.0");
assert_bitand("false", 0.0);

trace("//0.0 & 0.0");
assert_bitand(0.0, 0.0);

trace("//NaN & 0.0");
assert_bitand(NaN, 0.0);

trace("//-0.0 & 0.0");
assert_bitand(-0.0, 0.0);

trace("//Infinity & 0.0");
assert_bitand(Infinity, 0.0);

trace("//1.0 & 0.0");
assert_bitand(1.0, 0.0);

trace("//-1.0 & 0.0");
assert_bitand(-1.0, 0.0);

trace("//0xFF1306 & 0.0");
assert_bitand(0xFF1306, 0.0);

trace("//new Object() & 0.0");
assert_bitand({}, 0.0);

trace("//\"0.0\" & 0.0");
assert_bitand("0.0", 0.0);

trace("//\"NaN\" & 0.0");
assert_bitand("NaN", 0.0);

trace("//\"-0.0\" & 0.0");
assert_bitand("-0.0", 0.0);

trace("//\"Infinity\" & 0.0");
assert_bitand("Infinity", 0.0);

trace("//\"1.0\" & 0.0");
assert_bitand("1.0", 0.0);

trace("//\"-1.0\" & 0.0");
assert_bitand("-1.0", 0.0);

trace("//\"0xFF1306\" & 0.0");
assert_bitand("0xFF1306", 0.0);

trace("//true & NaN");
assert_bitand(true, NaN);

trace("//false & NaN");
assert_bitand(false, NaN);

trace("//null & NaN");
assert_bitand(null, NaN);

trace("//undefined & NaN");
assert_bitand(undefined, NaN);

trace("//\"\" & NaN");
assert_bitand("", NaN);

trace("//\"str\" & NaN");
assert_bitand("str", NaN);

trace("//\"true\" & NaN");
assert_bitand("true", NaN);

trace("//\"false\" & NaN");
assert_bitand("false", NaN);

trace("//0.0 & NaN");
assert_bitand(0.0, NaN);

trace("//NaN & NaN");
assert_bitand(NaN, NaN);

trace("//-0.0 & NaN");
assert_bitand(-0.0, NaN);

trace("//Infinity & NaN");
assert_bitand(Infinity, NaN);

trace("//1.0 & NaN");
assert_bitand(1.0, NaN);

trace("//-1.0 & NaN");
assert_bitand(-1.0, NaN);

trace("//0xFF1306 & NaN");
assert_bitand(0xFF1306, NaN);

trace("//new Object() & NaN");
assert_bitand({}, NaN);

trace("//\"0.0\" & NaN");
assert_bitand("0.0", NaN);

trace("//\"NaN\" & NaN");
assert_bitand("NaN", NaN);

trace("//\"-0.0\" & NaN");
assert_bitand("-0.0", NaN);

trace("//\"Infinity\" & NaN");
assert_bitand("Infinity", NaN);

trace("//\"1.0\" & NaN");
assert_bitand("1.0", NaN);

trace("//\"-1.0\" & NaN");
assert_bitand("-1.0", NaN);

trace("//\"0xFF1306\" & NaN");
assert_bitand("0xFF1306", NaN);

trace("//true & -0.0");
assert_bitand(true, -0.0);

trace("//false & -0.0");
assert_bitand(false, -0.0);

trace("//null & -0.0");
assert_bitand(null, -0.0);

trace("//undefined & -0.0");
assert_bitand(undefined, -0.0);

trace("//\"\" & -0.0");
assert_bitand("", -0.0);

trace("//\"str\" & -0.0");
assert_bitand("str", -0.0);

trace("//\"true\" & -0.0");
assert_bitand("true", -0.0);

trace("//\"false\" & -0.0");
assert_bitand("false", -0.0);

trace("//0.0 & -0.0");
assert_bitand(0.0, -0.0);

trace("//NaN & -0.0");
assert_bitand(NaN, -0.0);

trace("//-0.0 & -0.0");
assert_bitand(-0.0, -0.0);

trace("//Infinity & -0.0");
assert_bitand(Infinity, -0.0);

trace("//1.0 & -0.0");
assert_bitand(1.0, -0.0);

trace("//-1.0 & -0.0");
assert_bitand(-1.0, -0.0);

trace("//0xFF1306 & -0.0");
assert_bitand(0xFF1306, -0.0);

trace("//new Object() & -0.0");
assert_bitand({}, -0.0);

trace("//\"0.0\" & -0.0");
assert_bitand("0.0", -0.0);

trace("//\"NaN\" & -0.0");
assert_bitand("NaN", -0.0);

trace("//\"-0.0\" & -0.0");
assert_bitand("-0.0", -0.0);

trace("//\"Infinity\" & -0.0");
assert_bitand("Infinity", -0.0);

trace("//\"1.0\" & -0.0");
assert_bitand("1.0", -0.0);

trace("//\"-1.0\" & -0.0");
assert_bitand("-1.0", -0.0);

trace("//\"0xFF1306\" & -0.0");
assert_bitand("0xFF1306", -0.0);

trace("//true & Infinity");
assert_bitand(true, Infinity);

trace("//false & Infinity");
assert_bitand(false, Infinity);

trace("//null & Infinity");
assert_bitand(null, Infinity);

trace("//undefined & Infinity");
assert_bitand(undefined, Infinity);

trace("//\"\" & Infinity");
assert_bitand("", Infinity);

trace("//\"str\" & Infinity");
assert_bitand("str", Infinity);

trace("//\"true\" & Infinity");
assert_bitand("true", Infinity);

trace("//\"false\" & Infinity");
assert_bitand("false", Infinity);

trace("//0.0 & Infinity");
assert_bitand(0.0, Infinity);

trace("//NaN & Infinity");
assert_bitand(NaN, Infinity);

trace("//-0.0 & Infinity");
assert_bitand(-0.0, Infinity);

trace("//Infinity & Infinity");
assert_bitand(Infinity, Infinity);

trace("//1.0 & Infinity");
assert_bitand(1.0, Infinity);

trace("//-1.0 & Infinity");
assert_bitand(-1.0, Infinity);

trace("//0xFF1306 & Infinity");
assert_bitand(0xFF1306, Infinity);

trace("//new Object() & Infinity");
assert_bitand({}, Infinity);

trace("//\"0.0\" & Infinity");
assert_bitand("0.0", Infinity);

trace("//\"NaN\" & Infinity");
assert_bitand("NaN", Infinity);

trace("//\"-0.0\" & Infinity");
assert_bitand("-0.0", Infinity);

trace("//\"Infinity\" & Infinity");
assert_bitand("Infinity", Infinity);

trace("//\"1.0\" & Infinity");
assert_bitand("1.0", Infinity);

trace("//\"-1.0\" & Infinity");
assert_bitand("-1.0", Infinity);

trace("//\"0xFF1306\" & Infinity");
assert_bitand("0xFF1306", Infinity);

trace("//true & 1.0");
assert_bitand(true, 1.0);

trace("//false & 1.0");
assert_bitand(false, 1.0);

trace("//null & 1.0");
assert_bitand(null, 1.0);

trace("//undefined & 1.0");
assert_bitand(undefined, 1.0);

trace("//\"\" & 1.0");
assert_bitand("", 1.0);

trace("//\"str\" & 1.0");
assert_bitand("str", 1.0);

trace("//\"true\" & 1.0");
assert_bitand("true", 1.0);

trace("//\"false\" & 1.0");
assert_bitand("false", 1.0);

trace("//0.0 & 1.0");
assert_bitand(0.0, 1.0);

trace("//NaN & 1.0");
assert_bitand(NaN, 1.0);

trace("//-0.0 & 1.0");
assert_bitand(-0.0, 1.0);

trace("//Infinity & 1.0");
assert_bitand(Infinity, 1.0);

trace("//1.0 & 1.0");
assert_bitand(1.0, 1.0);

trace("//-1.0 & 1.0");
assert_bitand(-1.0, 1.0);

trace("//0xFF1306 & 1.0");
assert_bitand(0xFF1306, 1.0);

trace("//new Object() & 1.0");
assert_bitand({}, 1.0);

trace("//\"0.0\" & 1.0");
assert_bitand("0.0", 1.0);

trace("//\"NaN\" & 1.0");
assert_bitand("NaN", 1.0);

trace("//\"-0.0\" & 1.0");
assert_bitand("-0.0", 1.0);

trace("//\"Infinity\" & 1.0");
assert_bitand("Infinity", 1.0);

trace("//\"1.0\" & 1.0");
assert_bitand("1.0", 1.0);

trace("//\"-1.0\" & 1.0");
assert_bitand("-1.0", 1.0);

trace("//\"0xFF1306\" & 1.0");
assert_bitand("0xFF1306", 1.0);

trace("//true & -1.0");
assert_bitand(true, -1.0);

trace("//false & -1.0");
assert_bitand(false, -1.0);

trace("//null & -1.0");
assert_bitand(null, -1.0);

trace("//undefined & -1.0");
assert_bitand(undefined, -1.0);

trace("//\"\" & -1.0");
assert_bitand("", -1.0);

trace("//\"str\" & -1.0");
assert_bitand("str", -1.0);

trace("//\"true\" & -1.0");
assert_bitand("true", -1.0);

trace("//\"false\" & -1.0");
assert_bitand("false", -1.0);

trace("//0.0 & -1.0");
assert_bitand(0.0, -1.0);

trace("//NaN & -1.0");
assert_bitand(NaN, -1.0);

trace("//-0.0 & -1.0");
assert_bitand(-0.0, -1.0);

trace("//Infinity & -1.0");
assert_bitand(Infinity, -1.0);

trace("//1.0 & -1.0");
assert_bitand(1.0, -1.0);

trace("//-1.0 & -1.0");
assert_bitand(-1.0, -1.0);

trace("//0xFF1306 & -1.0");
assert_bitand(0xFF1306, -1.0);

trace("//new Object() & -1.0");
assert_bitand({}, -1.0);

trace("//\"0.0\" & -1.0");
assert_bitand("0.0", -1.0);

trace("//\"NaN\" & -1.0");
assert_bitand("NaN", -1.0);

trace("//\"-0.0\" & -1.0");
assert_bitand("-0.0", -1.0);

trace("//\"Infinity\" & -1.0");
assert_bitand("Infinity", -1.0);

trace("//\"1.0\" & -1.0");
assert_bitand("1.0", -1.0);

trace("//\"-1.0\" & -1.0");
assert_bitand("-1.0", -1.0);

trace("//\"0xFF1306\" & -1.0");
assert_bitand("0xFF1306", -1.0);

trace("//true & 0xFF1306");
assert_bitand(true, 0xFF1306);

trace("//false & 0xFF1306");
assert_bitand(false, 0xFF1306);

trace("//null & 0xFF1306");
assert_bitand(null, 0xFF1306);

trace("//undefined & 0xFF1306");
assert_bitand(undefined, 0xFF1306);

trace("//\"\" & 0xFF1306");
assert_bitand("", 0xFF1306);

trace("//\"str\" & 0xFF1306");
assert_bitand("str", 0xFF1306);

trace("//\"true\" & 0xFF1306");
assert_bitand("true", 0xFF1306);

trace("//\"false\" & 0xFF1306");
assert_bitand("false", 0xFF1306);

trace("//0.0 & 0xFF1306");
assert_bitand(0.0, 0xFF1306);

trace("//NaN & 0xFF1306");
assert_bitand(NaN, 0xFF1306);

trace("//-0.0 & 0xFF1306");
assert_bitand(-0.0, 0xFF1306);

trace("//Infinity & 0xFF1306");
assert_bitand(Infinity, 0xFF1306);

trace("//1.0 & 0xFF1306");
assert_bitand(1.0, 0xFF1306);

trace("//-1.0 & 0xFF1306");
assert_bitand(-1.0, 0xFF1306);

trace("//0xFF1306 & 0xFF1306");
assert_bitand(0xFF1306, 0xFF1306);

trace("//new Object() & 0xFF1306");
assert_bitand({}, 0xFF1306);

trace("//\"0.0\" & 0xFF1306");
assert_bitand("0.0", 0xFF1306);

trace("//\"NaN\" & 0xFF1306");
assert_bitand("NaN", 0xFF1306);

trace("//\"-0.0\" & 0xFF1306");
assert_bitand("-0.0", 0xFF1306);

trace("//\"Infinity\" & 0xFF1306");
assert_bitand("Infinity", 0xFF1306);

trace("//\"1.0\" & 0xFF1306");
assert_bitand("1.0", 0xFF1306);

trace("//\"-1.0\" & 0xFF1306");
assert_bitand("-1.0", 0xFF1306);

trace("//\"0xFF1306\" & 0xFF1306");
assert_bitand("0xFF1306", 0xFF1306);

trace("//true & new Object()");
assert_bitand(true, {});

trace("//false & new Object()");
assert_bitand(false, {});

trace("//null & new Object()");
assert_bitand(null, {});

trace("//undefined & new Object()");
assert_bitand(undefined, {});

trace("//\"\" & new Object()");
assert_bitand("", {});

trace("//\"str\" & new Object()");
assert_bitand("str", {});

trace("//\"true\" & new Object()");
assert_bitand("true", {});

trace("//\"false\" & new Object()");
assert_bitand("false", {});

trace("//0.0 & new Object()");
assert_bitand(0.0, {});

trace("//NaN & new Object()");
assert_bitand(NaN, {});

trace("//-0.0 & new Object()");
assert_bitand(-0.0, {});

trace("//Infinity & new Object()");
assert_bitand(Infinity, {});

trace("//1.0 & new Object()");
assert_bitand(1.0, {});

trace("//-1.0 & new Object()");
assert_bitand(-1.0, {});

trace("//0xFF1306 & new Object()");
assert_bitand(0xFF1306, {});

trace("//new Object() & new Object()");
assert_bitand({}, {});

trace("//\"0.0\" & new Object()");
assert_bitand("0.0", {});

trace("//\"NaN\" & new Object()");
assert_bitand("NaN", {});

trace("//\"-0.0\" & new Object()");
assert_bitand("-0.0", {});

trace("//\"Infinity\" & new Object()");
assert_bitand("Infinity", {});

trace("//\"1.0\" & new Object()");
assert_bitand("1.0", {});

trace("//\"-1.0\" & new Object()");
assert_bitand("-1.0", {});

trace("//\"0xFF1306\" & new Object()");
assert_bitand("0xFF1306", {});

trace("//true & \"0.0\"");
assert_bitand(true, "0.0");

trace("//false & \"0.0\"");
assert_bitand(false, "0.0");

trace("//null & \"0.0\"");
assert_bitand(null, "0.0");

trace("//undefined & \"0.0\"");
assert_bitand(undefined, "0.0");

trace("//\"\" & \"0.0\"");
assert_bitand("", "0.0");

trace("//\"str\" & \"0.0\"");
assert_bitand("str", "0.0");

trace("//\"true\" & \"0.0\"");
assert_bitand("true", "0.0");

trace("//\"false\" & \"0.0\"");
assert_bitand("false", "0.0");

trace("//0.0 & \"0.0\"");
assert_bitand(0.0, "0.0");

trace("//NaN & \"0.0\"");
assert_bitand(NaN, "0.0");

trace("//-0.0 & \"0.0\"");
assert_bitand(-0.0, "0.0");

trace("//Infinity & \"0.0\"");
assert_bitand(Infinity, "0.0");

trace("//1.0 & \"0.0\"");
assert_bitand(1.0, "0.0");

trace("//-1.0 & \"0.0\"");
assert_bitand(-1.0, "0.0");

trace("//0xFF1306 & \"0.0\"");
assert_bitand(0xFF1306, "0.0");

trace("//new Object() & \"0.0\"");
assert_bitand({}, "0.0");

trace("//\"0.0\" & \"0.0\"");
assert_bitand("0.0", "0.0");

trace("//\"NaN\" & \"0.0\"");
assert_bitand("NaN", "0.0");

trace("//\"-0.0\" & \"0.0\"");
assert_bitand("-0.0", "0.0");

trace("//\"Infinity\" & \"0.0\"");
assert_bitand("Infinity", "0.0");

trace("//\"1.0\" & \"0.0\"");
assert_bitand("1.0", "0.0");

trace("//\"-1.0\" & \"0.0\"");
assert_bitand("-1.0", "0.0");

trace("//\"0xFF1306\" & \"0.0\"");
assert_bitand("0xFF1306", "0.0");

trace("//true & \"NaN\"");
assert_bitand(true, "NaN");

trace("//false & \"NaN\"");
assert_bitand(false, "NaN");

trace("//null & \"NaN\"");
assert_bitand(null, "NaN");

trace("//undefined & \"NaN\"");
assert_bitand(undefined, "NaN");

trace("//\"\" & \"NaN\"");
assert_bitand("", "NaN");

trace("//\"str\" & \"NaN\"");
assert_bitand("str", "NaN");

trace("//\"true\" & \"NaN\"");
assert_bitand("true", "NaN");

trace("//\"false\" & \"NaN\"");
assert_bitand("false", "NaN");

trace("//0.0 & \"NaN\"");
assert_bitand(0.0, "NaN");

trace("//NaN & \"NaN\"");
assert_bitand(NaN, "NaN");

trace("//-0.0 & \"NaN\"");
assert_bitand(-0.0, "NaN");

trace("//Infinity & \"NaN\"");
assert_bitand(Infinity, "NaN");

trace("//1.0 & \"NaN\"");
assert_bitand(1.0, "NaN");

trace("//-1.0 & \"NaN\"");
assert_bitand(-1.0, "NaN");

trace("//0xFF1306 & \"NaN\"");
assert_bitand(0xFF1306, "NaN");

trace("//new Object() & \"NaN\"");
assert_bitand({}, "NaN");

trace("//\"0.0\" & \"NaN\"");
assert_bitand("0.0", "NaN");

trace("//\"NaN\" & \"NaN\"");
assert_bitand("NaN", "NaN");

trace("//\"-0.0\" & \"NaN\"");
assert_bitand("-0.0", "NaN");

trace("//\"Infinity\" & \"NaN\"");
assert_bitand("Infinity", "NaN");

trace("//\"1.0\" & \"NaN\"");
assert_bitand("1.0", "NaN");

trace("//\"-1.0\" & \"NaN\"");
assert_bitand("-1.0", "NaN");

trace("//\"0xFF1306\" & \"NaN\"");
assert_bitand("0xFF1306", "NaN");

trace("//true & \"-0.0\"");
assert_bitand(true, "-0.0");

trace("//false & \"-0.0\"");
assert_bitand(false, "-0.0");

trace("//null & \"-0.0\"");
assert_bitand(null, "-0.0");

trace("//undefined & \"-0.0\"");
assert_bitand(undefined, "-0.0");

trace("//\"\" & \"-0.0\"");
assert_bitand("", "-0.0");

trace("//\"str\" & \"-0.0\"");
assert_bitand("str", "-0.0");

trace("//\"true\" & \"-0.0\"");
assert_bitand("true", "-0.0");

trace("//\"false\" & \"-0.0\"");
assert_bitand("false", "-0.0");

trace("//0.0 & \"-0.0\"");
assert_bitand(0.0, "-0.0");

trace("//NaN & \"-0.0\"");
assert_bitand(NaN, "-0.0");

trace("//-0.0 & \"-0.0\"");
assert_bitand(-0.0, "-0.0");

trace("//Infinity & \"-0.0\"");
assert_bitand(Infinity, "-0.0");

trace("//1.0 & \"-0.0\"");
assert_bitand(1.0, "-0.0");

trace("//-1.0 & \"-0.0\"");
assert_bitand(-1.0, "-0.0");

trace("//0xFF1306 & \"-0.0\"");
assert_bitand(0xFF1306, "-0.0");

trace("//new Object() & \"-0.0\"");
assert_bitand({}, "-0.0");

trace("//\"0.0\" & \"-0.0\"");
assert_bitand("0.0", "-0.0");

trace("//\"NaN\" & \"-0.0\"");
assert_bitand("NaN", "-0.0");

trace("//\"-0.0\" & \"-0.0\"");
assert_bitand("-0.0", "-0.0");

trace("//\"Infinity\" & \"-0.0\"");
assert_bitand("Infinity", "-0.0");

trace("//\"1.0\" & \"-0.0\"");
assert_bitand("1.0", "-0.0");

trace("//\"-1.0\" & \"-0.0\"");
assert_bitand("-1.0", "-0.0");

trace("//\"0xFF1306\" & \"-0.0\"");
assert_bitand("0xFF1306", "-0.0");

trace("//true & \"Infinity\"");
assert_bitand(true, "Infinity");

trace("//false & \"Infinity\"");
assert_bitand(false, "Infinity");

trace("//null & \"Infinity\"");
assert_bitand(null, "Infinity");

trace("//undefined & \"Infinity\"");
assert_bitand(undefined, "Infinity");

trace("//\"\" & \"Infinity\"");
assert_bitand("", "Infinity");

trace("//\"str\" & \"Infinity\"");
assert_bitand("str", "Infinity");

trace("//\"true\" & \"Infinity\"");
assert_bitand("true", "Infinity");

trace("//\"false\" & \"Infinity\"");
assert_bitand("false", "Infinity");

trace("//0.0 & \"Infinity\"");
assert_bitand(0.0, "Infinity");

trace("//NaN & \"Infinity\"");
assert_bitand(NaN, "Infinity");

trace("//-0.0 & \"Infinity\"");
assert_bitand(-0.0, "Infinity");

trace("//Infinity & \"Infinity\"");
assert_bitand(Infinity, "Infinity");

trace("//1.0 & \"Infinity\"");
assert_bitand(1.0, "Infinity");

trace("//-1.0 & \"Infinity\"");
assert_bitand(-1.0, "Infinity");

trace("//0xFF1306 & \"Infinity\"");
assert_bitand(0xFF1306, "Infinity");

trace("//new Object() & \"Infinity\"");
assert_bitand({}, "Infinity");

trace("//\"0.0\" & \"Infinity\"");
assert_bitand("0.0", "Infinity");

trace("//\"NaN\" & \"Infinity\"");
assert_bitand("NaN", "Infinity");

trace("//\"-0.0\" & \"Infinity\"");
assert_bitand("-0.0", "Infinity");

trace("//\"Infinity\" & \"Infinity\"");
assert_bitand("Infinity", "Infinity");

trace("//\"1.0\" & \"Infinity\"");
assert_bitand("1.0", "Infinity");

trace("//\"-1.0\" & \"Infinity\"");
assert_bitand("-1.0", "Infinity");

trace("//\"0xFF1306\" & \"Infinity\"");
assert_bitand("0xFF1306", "Infinity");

trace("//true & \"1.0\"");
assert_bitand(true, "1.0");

trace("//false & \"1.0\"");
assert_bitand(false, "1.0");

trace("//null & \"1.0\"");
assert_bitand(null, "1.0");

trace("//undefined & \"1.0\"");
assert_bitand(undefined, "1.0");

trace("//\"\" & \"1.0\"");
assert_bitand("", "1.0");

trace("//\"str\" & \"1.0\"");
assert_bitand("str", "1.0");

trace("//\"true\" & \"1.0\"");
assert_bitand("true", "1.0");

trace("//\"false\" & \"1.0\"");
assert_bitand("false", "1.0");

trace("//0.0 & \"1.0\"");
assert_bitand(0.0, "1.0");

trace("//NaN & \"1.0\"");
assert_bitand(NaN, "1.0");

trace("//-0.0 & \"1.0\"");
assert_bitand(-0.0, "1.0");

trace("//Infinity & \"1.0\"");
assert_bitand(Infinity, "1.0");

trace("//1.0 & \"1.0\"");
assert_bitand(1.0, "1.0");

trace("//-1.0 & \"1.0\"");
assert_bitand(-1.0, "1.0");

trace("//0xFF1306 & \"1.0\"");
assert_bitand(0xFF1306, "1.0");

trace("//new Object() & \"1.0\"");
assert_bitand({}, "1.0");

trace("//\"0.0\" & \"1.0\"");
assert_bitand("0.0", "1.0");

trace("//\"NaN\" & \"1.0\"");
assert_bitand("NaN", "1.0");

trace("//\"-0.0\" & \"1.0\"");
assert_bitand("-0.0", "1.0");

trace("//\"Infinity\" & \"1.0\"");
assert_bitand("Infinity", "1.0");

trace("//\"1.0\" & \"1.0\"");
assert_bitand("1.0", "1.0");

trace("//\"-1.0\" & \"1.0\"");
assert_bitand("-1.0", "1.0");

trace("//\"0xFF1306\" & \"1.0\"");
assert_bitand("0xFF1306", "1.0");

trace("//true & \"-1.0\"");
assert_bitand(true, "-1.0");

trace("//false & \"-1.0\"");
assert_bitand(false, "-1.0");

trace("//null & \"-1.0\"");
assert_bitand(null, "-1.0");

trace("//undefined & \"-1.0\"");
assert_bitand(undefined, "-1.0");

trace("//\"\" & \"-1.0\"");
assert_bitand("", "-1.0");

trace("//\"str\" & \"-1.0\"");
assert_bitand("str", "-1.0");

trace("//\"true\" & \"-1.0\"");
assert_bitand("true", "-1.0");

trace("//\"false\" & \"-1.0\"");
assert_bitand("false", "-1.0");

trace("//0.0 & \"-1.0\"");
assert_bitand(0.0, "-1.0");

trace("//NaN & \"-1.0\"");
assert_bitand(NaN, "-1.0");

trace("//-0.0 & \"-1.0\"");
assert_bitand(-0.0, "-1.0");

trace("//Infinity & \"-1.0\"");
assert_bitand(Infinity, "-1.0");

trace("//1.0 & \"-1.0\"");
assert_bitand(1.0, "-1.0");

trace("//-1.0 & \"-1.0\"");
assert_bitand(-1.0, "-1.0");

trace("//0xFF1306 & \"-1.0\"");
assert_bitand(0xFF1306, "-1.0");

trace("//new Object() & \"-1.0\"");
assert_bitand({}, "-1.0");

trace("//\"0.0\" & \"-1.0\"");
assert_bitand("0.0", "-1.0");

trace("//\"NaN\" & \"-1.0\"");
assert_bitand("NaN", "-1.0");

trace("//\"-0.0\" & \"-1.0\"");
assert_bitand("-0.0", "-1.0");

trace("//\"Infinity\" & \"-1.0\"");
assert_bitand("Infinity", "-1.0");

trace("//\"1.0\" & \"-1.0\"");
assert_bitand("1.0", "-1.0");

trace("//\"-1.0\" & \"-1.0\"");
assert_bitand("-1.0", "-1.0");

trace("//\"0xFF1306\" & \"-1.0\"");
assert_bitand("0xFF1306", "-1.0");

trace("//true & \"0xFF1306\"");
assert_bitand(true, "0xFF1306");

trace("//false & \"0xFF1306\"");
assert_bitand(false, "0xFF1306");

trace("//null & \"0xFF1306\"");
assert_bitand(null, "0xFF1306");

trace("//undefined & \"0xFF1306\"");
assert_bitand(undefined, "0xFF1306");

trace("//\"\" & \"0xFF1306\"");
assert_bitand("", "0xFF1306");

trace("//\"str\" & \"0xFF1306\"");
assert_bitand("str", "0xFF1306");

trace("//\"true\" & \"0xFF1306\"");
assert_bitand("true", "0xFF1306");

trace("//\"false\" & \"0xFF1306\"");
assert_bitand("false", "0xFF1306");

trace("//0.0 & \"0xFF1306\"");
assert_bitand(0.0, "0xFF1306");

trace("//NaN & \"0xFF1306\"");
assert_bitand(NaN, "0xFF1306");

trace("//-0.0 & \"0xFF1306\"");
assert_bitand(-0.0, "0xFF1306");

trace("//Infinity & \"0xFF1306\"");
assert_bitand(Infinity, "0xFF1306");

trace("//1.0 & \"0xFF1306\"");
assert_bitand(1.0, "0xFF1306");

trace("//-1.0 & \"0xFF1306\"");
assert_bitand(-1.0, "0xFF1306");

trace("//0xFF1306 & \"0xFF1306\"");
assert_bitand(0xFF1306, "0xFF1306");

trace("//new Object() & \"0xFF1306\"");
assert_bitand({}, "0xFF1306");

trace("//\"0.0\" & \"0xFF1306\"");
assert_bitand("0.0", "0xFF1306");

trace("//\"NaN\" & \"0xFF1306\"");
assert_bitand("NaN", "0xFF1306");

trace("//\"-0.0\" & \"0xFF1306\"");
assert_bitand("-0.0", "0xFF1306");

trace("//\"Infinity\" & \"0xFF1306\"");
assert_bitand("Infinity", "0xFF1306");

trace("//\"1.0\" & \"0xFF1306\"");
assert_bitand("1.0", "0xFF1306");

trace("//\"-1.0\" & \"0xFF1306\"");
assert_bitand("-1.0", "0xFF1306");

trace("//\"0xFF1306\" & \"0xFF1306\"");
assert_bitand("0xFF1306", "0xFF1306");