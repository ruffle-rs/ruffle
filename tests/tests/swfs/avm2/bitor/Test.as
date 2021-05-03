package {
	public class Test {
	}
}

function assert_bitor(val1, val2) {
	trace(val1 | val2);
}

trace("//true | true");
assert_bitor(true, true);

trace("//false | true");
assert_bitor(false, true);

trace("//null | true");
assert_bitor(null, true);

trace("//undefined | true");
assert_bitor(undefined, true);

trace("//\"\" | true");
assert_bitor("", true);

trace("//\"str\" | true");
assert_bitor("str", true);

trace("//\"true\" | true");
assert_bitor("true", true);

trace("//\"false\" | true");
assert_bitor("false", true);

trace("//0.0 | true");
assert_bitor(0.0, true);

trace("//NaN | true");
assert_bitor(NaN, true);

trace("//-0.0 | true");
assert_bitor(-0.0, true);

trace("//Infinity | true");
assert_bitor(Infinity, true);

trace("//1.0 | true");
assert_bitor(1.0, true);

trace("//-1.0 | true");
assert_bitor(-1.0, true);

trace("//0xFF1306 | true");
assert_bitor(0xFF1306, true);

trace("//new Object() | true");
assert_bitor({}, true);

trace("//\"0.0\" | true");
assert_bitor("0.0", true);

trace("//\"NaN\" | true");
assert_bitor("NaN", true);

trace("//\"-0.0\" | true");
assert_bitor("-0.0", true);

trace("//\"Infinity\" | true");
assert_bitor("Infinity", true);

trace("//\"1.0\" | true");
assert_bitor("1.0", true);

trace("//\"-1.0\" | true");
assert_bitor("-1.0", true);

trace("//\"0xFF1306\" | true");
assert_bitor("0xFF1306", true);

trace("//true | false");
assert_bitor(true, false);

trace("//false | false");
assert_bitor(false, false);

trace("//null | false");
assert_bitor(null, false);

trace("//undefined | false");
assert_bitor(undefined, false);

trace("//\"\" | false");
assert_bitor("", false);

trace("//\"str\" | false");
assert_bitor("str", false);

trace("//\"true\" | false");
assert_bitor("true", false);

trace("//\"false\" | false");
assert_bitor("false", false);

trace("//0.0 | false");
assert_bitor(0.0, false);

trace("//NaN | false");
assert_bitor(NaN, false);

trace("//-0.0 | false");
assert_bitor(-0.0, false);

trace("//Infinity | false");
assert_bitor(Infinity, false);

trace("//1.0 | false");
assert_bitor(1.0, false);

trace("//-1.0 | false");
assert_bitor(-1.0, false);

trace("//0xFF1306 | false");
assert_bitor(0xFF1306, false);

trace("//new Object() | false");
assert_bitor({}, false);

trace("//\"0.0\" | false");
assert_bitor("0.0", false);

trace("//\"NaN\" | false");
assert_bitor("NaN", false);

trace("//\"-0.0\" | false");
assert_bitor("-0.0", false);

trace("//\"Infinity\" | false");
assert_bitor("Infinity", false);

trace("//\"1.0\" | false");
assert_bitor("1.0", false);

trace("//\"-1.0\" | false");
assert_bitor("-1.0", false);

trace("//\"0xFF1306\" | false");
assert_bitor("0xFF1306", false);
trace("//true | null");
assert_bitor(true, null);

trace("//false | null");
assert_bitor(false, null);

trace("//null | null");
assert_bitor(null, null);

trace("//undefined | null");
assert_bitor(undefined, null);

trace("//\"\" | null");
assert_bitor("", null);

trace("//\"str\" | null");
assert_bitor("str", null);

trace("//\"true\" | null");
assert_bitor("true", null);

trace("//\"false\" | null");
assert_bitor("false", null);

trace("//0.0 | null");
assert_bitor(0.0, null);

trace("//NaN | null");
assert_bitor(NaN, null);

trace("//-0.0 | null");
assert_bitor(-0.0, null);

trace("//Infinity | null");
assert_bitor(Infinity, null);

trace("//1.0 | null");
assert_bitor(1.0, null);

trace("//-1.0 | null");
assert_bitor(-1.0, null);

trace("//0xFF1306 | null");
assert_bitor(0xFF1306, null);

trace("//new Object() | null");
assert_bitor({}, null);

trace("//\"0.0\" | null");
assert_bitor("0.0", null);

trace("//\"NaN\" | null");
assert_bitor("NaN", null);

trace("//\"-0.0\" | null");
assert_bitor("-0.0", null);

trace("//\"Infinity\" | null");
assert_bitor("Infinity", null);

trace("//\"1.0\" | null");
assert_bitor("1.0", null);

trace("//\"-1.0\" | null");
assert_bitor("-1.0", null);

trace("//\"0xFF1306\" | null");
assert_bitor("0xFF1306", null);

trace("//true | undefined");
assert_bitor(true, undefined);

trace("//false | undefined");
assert_bitor(false, undefined);

trace("//null | undefined");
assert_bitor(null, undefined);

trace("//undefined | undefined");
assert_bitor(undefined, undefined);

trace("//\"\" | undefined");
assert_bitor("", undefined);

trace("//\"str\" | undefined");
assert_bitor("str", undefined);

trace("//\"true\" | undefined");
assert_bitor("true", undefined);

trace("//\"false\" | undefined");
assert_bitor("false", undefined);

trace("//0.0 | undefined");
assert_bitor(0.0, undefined);

trace("//NaN | undefined");
assert_bitor(NaN, undefined);

trace("//-0.0 | undefined");
assert_bitor(-0.0, undefined);

trace("//Infinity | undefined");
assert_bitor(Infinity, undefined);

trace("//1.0 | undefined");
assert_bitor(1.0, undefined);

trace("//-1.0 | undefined");
assert_bitor(-1.0, undefined);

trace("//0xFF1306 | undefined");
assert_bitor(0xFF1306, undefined);

trace("//new Object() | undefined");
assert_bitor({}, undefined);

trace("//\"0.0\" | undefined");
assert_bitor("0.0", undefined);

trace("//\"NaN\" | undefined");
assert_bitor("NaN", undefined);

trace("//\"-0.0\" | undefined");
assert_bitor("-0.0", undefined);

trace("//\"Infinity\" | undefined");
assert_bitor("Infinity", undefined);

trace("//\"1.0\" | undefined");
assert_bitor("1.0", undefined);

trace("//\"-1.0\" | undefined");
assert_bitor("-1.0", undefined);

trace("//\"0xFF1306\" | undefined");
assert_bitor("0xFF1306", undefined);

trace("//true | \"\"");
assert_bitor(true, "");

trace("//false | \"\"");
assert_bitor(false, "");

trace("//null | \"\"");
assert_bitor(null, "");

trace("//undefined | \"\"");
assert_bitor(undefined, "");

trace("//\"\" | \"\"");
assert_bitor("", "");

trace("//\"str\" | \"\"");
assert_bitor("str", "");

trace("//\"true\" | \"\"");
assert_bitor("true", "");

trace("//\"false\" | \"\"");
assert_bitor("false", "");

trace("//0.0 | \"\"");
assert_bitor(0.0, "");

trace("//NaN | \"\"");
assert_bitor(NaN, "");

trace("//-0.0 | \"\"");
assert_bitor(-0.0, "");

trace("//Infinity | \"\"");
assert_bitor(Infinity, "");

trace("//1.0 | \"\"");
assert_bitor(1.0, "");

trace("//-1.0 | \"\"");
assert_bitor(-1.0, "");

trace("//0xFF1306 | \"\"");
assert_bitor(0xFF1306, "");

trace("//new Object() | \"\"");
assert_bitor({}, "");

trace("//\"0.0\" | \"\"");
assert_bitor("0.0", "");

trace("//\"NaN\" | \"\"");
assert_bitor("NaN", "");

trace("//\"-0.0\" | \"\"");
assert_bitor("-0.0", "");

trace("//\"Infinity\" | \"\"");
assert_bitor("Infinity", "");

trace("//\"1.0\" | \"\"");
assert_bitor("1.0", "");

trace("//\"-1.0\" | \"\"");
assert_bitor("-1.0", "");

trace("//\"0xFF1306\" | \"\"");
assert_bitor("0xFF1306", "");

trace("//true | \"str\"");
assert_bitor(true, "str");

trace("//false | \"str\"");
assert_bitor(false, "str");

trace("//null | \"str\"");
assert_bitor(null, "str");

trace("//undefined | \"str\"");
assert_bitor(undefined, "str");

trace("//\"\" | \"str\"");
assert_bitor("", "str");

trace("//\"str\" | \"str\"");
assert_bitor("str", "str");

trace("//\"true\" | \"str\"");
assert_bitor("true", "str");

trace("//\"false\" | \"str\"");
assert_bitor("false", "str");

trace("//0.0 | \"str\"");
assert_bitor(0.0, "str");

trace("//NaN | \"str\"");
assert_bitor(NaN, "str");

trace("//-0.0 | \"str\"");
assert_bitor(-0.0, "str");

trace("//Infinity | \"str\"");
assert_bitor(Infinity, "str");

trace("//1.0 | \"str\"");
assert_bitor(1.0, "str");

trace("//-1.0 | \"str\"");
assert_bitor(-1.0, "str");

trace("//0xFF1306 | \"str\"");
assert_bitor(0xFF1306, "str");

trace("//new Object() | \"str\"");
assert_bitor({}, "str");

trace("//\"0.0\" | \"str\"");
assert_bitor("0.0", "str");

trace("//\"NaN\" | \"str\"");
assert_bitor("NaN", "str");

trace("//\"-0.0\" | \"str\"");
assert_bitor("-0.0", "str");

trace("//\"Infinity\" | \"str\"");
assert_bitor("Infinity", "str");

trace("//\"1.0\" | \"str\"");
assert_bitor("1.0", "str");

trace("//\"-1.0\" | \"str\"");
assert_bitor("-1.0", "str");

trace("//\"0xFF1306\" | \"str\"");
assert_bitor("0xFF1306", "str");

trace("//true | \"true\"");
assert_bitor(true, "true");

trace("//false | \"true\"");
assert_bitor(false, "true");

trace("//null | \"true\"");
assert_bitor(null, "true");

trace("//undefined | \"true\"");
assert_bitor(undefined, "true");

trace("//\"\" | \"true\"");
assert_bitor("", "true");

trace("//\"str\" | \"true\"");
assert_bitor("str", "true");

trace("//\"true\" | \"true\"");
assert_bitor("true", "true");

trace("//\"false\" | \"true\"");
assert_bitor("false", "true");

trace("//0.0 | \"true\"");
assert_bitor(0.0, "true");

trace("//NaN | \"true\"");
assert_bitor(NaN, "true");

trace("//-0.0 | \"true\"");
assert_bitor(-0.0, "true");

trace("//Infinity | \"true\"");
assert_bitor(Infinity, "true");

trace("//1.0 | \"true\"");
assert_bitor(1.0, "true");

trace("//-1.0 | \"true\"");
assert_bitor(-1.0, "true");

trace("//0xFF1306 | \"true\"");
assert_bitor(0xFF1306, "true");

trace("//new Object() | \"true\"");
assert_bitor({}, "true");

trace("//\"0.0\" | \"true\"");
assert_bitor("0.0", "true");

trace("//\"NaN\" | \"true\"");
assert_bitor("NaN", "true");

trace("//\"-0.0\" | \"true\"");
assert_bitor("-0.0", "true");

trace("//\"Infinity\" | \"true\"");
assert_bitor("Infinity", "true");

trace("//\"1.0\" | \"true\"");
assert_bitor("1.0", "true");

trace("//\"-1.0\" | \"true\"");
assert_bitor("-1.0", "true");

trace("//\"0xFF1306\" | \"true\"");
assert_bitor("0xFF1306", "true");

trace("//true | \"false\"");
assert_bitor(true, "false");

trace("//false | \"false\"");
assert_bitor(false, "false");

trace("//null | \"false\"");
assert_bitor(null, "false");

trace("//undefined | \"false\"");
assert_bitor(undefined, "false");

trace("//\"\" | \"false\"");
assert_bitor("", "false");

trace("//\"str\" | \"false\"");
assert_bitor("str", "false");

trace("//\"true\" | \"false\"");
assert_bitor("true", "false");

trace("//\"false\" | \"false\"");
assert_bitor("false", "false");

trace("//0.0 | \"false\"");
assert_bitor(0.0, "false");

trace("//NaN | \"false\"");
assert_bitor(NaN, "false");

trace("//-0.0 | \"false\"");
assert_bitor(-0.0, "false");

trace("//Infinity | \"false\"");
assert_bitor(Infinity, "false");

trace("//1.0 | \"false\"");
assert_bitor(1.0, "false");

trace("//-1.0 | \"false\"");
assert_bitor(-1.0, "false");

trace("//0xFF1306 | \"false\"");
assert_bitor(0xFF1306, "false");

trace("//new Object() | \"false\"");
assert_bitor({}, "false");

trace("//\"0.0\" | \"false\"");
assert_bitor("0.0", "false");

trace("//\"NaN\" | \"false\"");
assert_bitor("NaN", "false");

trace("//\"-0.0\" | \"false\"");
assert_bitor("-0.0", "false");

trace("//\"Infinity\" | \"false\"");
assert_bitor("Infinity", "false");

trace("//\"1.0\" | \"false\"");
assert_bitor("1.0", "false");

trace("//\"-1.0\" | \"false\"");
assert_bitor("-1.0", "false");

trace("//\"0xFF1306\" | \"false\"");
assert_bitor("0xFF1306", "false");

trace("//true | 0.0");
assert_bitor(true, 0.0);

trace("//false | 0.0");
assert_bitor(false, 0.0);

trace("//null | 0.0");
assert_bitor(null, 0.0);

trace("//undefined | 0.0");
assert_bitor(undefined, 0.0);

trace("//\"\" | 0.0");
assert_bitor("", 0.0);

trace("//\"str\" | 0.0");
assert_bitor("str", 0.0);

trace("//\"true\" | 0.0");
assert_bitor("true", 0.0);

trace("//\"false\" | 0.0");
assert_bitor("false", 0.0);

trace("//0.0 | 0.0");
assert_bitor(0.0, 0.0);

trace("//NaN | 0.0");
assert_bitor(NaN, 0.0);

trace("//-0.0 | 0.0");
assert_bitor(-0.0, 0.0);

trace("//Infinity | 0.0");
assert_bitor(Infinity, 0.0);

trace("//1.0 | 0.0");
assert_bitor(1.0, 0.0);

trace("//-1.0 | 0.0");
assert_bitor(-1.0, 0.0);

trace("//0xFF1306 | 0.0");
assert_bitor(0xFF1306, 0.0);

trace("//new Object() | 0.0");
assert_bitor({}, 0.0);

trace("//\"0.0\" | 0.0");
assert_bitor("0.0", 0.0);

trace("//\"NaN\" | 0.0");
assert_bitor("NaN", 0.0);

trace("//\"-0.0\" | 0.0");
assert_bitor("-0.0", 0.0);

trace("//\"Infinity\" | 0.0");
assert_bitor("Infinity", 0.0);

trace("//\"1.0\" | 0.0");
assert_bitor("1.0", 0.0);

trace("//\"-1.0\" | 0.0");
assert_bitor("-1.0", 0.0);

trace("//\"0xFF1306\" | 0.0");
assert_bitor("0xFF1306", 0.0);

trace("//true | NaN");
assert_bitor(true, NaN);

trace("//false | NaN");
assert_bitor(false, NaN);

trace("//null | NaN");
assert_bitor(null, NaN);

trace("//undefined | NaN");
assert_bitor(undefined, NaN);

trace("//\"\" | NaN");
assert_bitor("", NaN);

trace("//\"str\" | NaN");
assert_bitor("str", NaN);

trace("//\"true\" | NaN");
assert_bitor("true", NaN);

trace("//\"false\" | NaN");
assert_bitor("false", NaN);

trace("//0.0 | NaN");
assert_bitor(0.0, NaN);

trace("//NaN | NaN");
assert_bitor(NaN, NaN);

trace("//-0.0 | NaN");
assert_bitor(-0.0, NaN);

trace("//Infinity | NaN");
assert_bitor(Infinity, NaN);

trace("//1.0 | NaN");
assert_bitor(1.0, NaN);

trace("//-1.0 | NaN");
assert_bitor(-1.0, NaN);

trace("//0xFF1306 | NaN");
assert_bitor(0xFF1306, NaN);

trace("//new Object() | NaN");
assert_bitor({}, NaN);

trace("//\"0.0\" | NaN");
assert_bitor("0.0", NaN);

trace("//\"NaN\" | NaN");
assert_bitor("NaN", NaN);

trace("//\"-0.0\" | NaN");
assert_bitor("-0.0", NaN);

trace("//\"Infinity\" | NaN");
assert_bitor("Infinity", NaN);

trace("//\"1.0\" | NaN");
assert_bitor("1.0", NaN);

trace("//\"-1.0\" | NaN");
assert_bitor("-1.0", NaN);

trace("//\"0xFF1306\" | NaN");
assert_bitor("0xFF1306", NaN);

trace("//true | -0.0");
assert_bitor(true, -0.0);

trace("//false | -0.0");
assert_bitor(false, -0.0);

trace("//null | -0.0");
assert_bitor(null, -0.0);

trace("//undefined | -0.0");
assert_bitor(undefined, -0.0);

trace("//\"\" | -0.0");
assert_bitor("", -0.0);

trace("//\"str\" | -0.0");
assert_bitor("str", -0.0);

trace("//\"true\" | -0.0");
assert_bitor("true", -0.0);

trace("//\"false\" | -0.0");
assert_bitor("false", -0.0);

trace("//0.0 | -0.0");
assert_bitor(0.0, -0.0);

trace("//NaN | -0.0");
assert_bitor(NaN, -0.0);

trace("//-0.0 | -0.0");
assert_bitor(-0.0, -0.0);

trace("//Infinity | -0.0");
assert_bitor(Infinity, -0.0);

trace("//1.0 | -0.0");
assert_bitor(1.0, -0.0);

trace("//-1.0 | -0.0");
assert_bitor(-1.0, -0.0);

trace("//0xFF1306 | -0.0");
assert_bitor(0xFF1306, -0.0);

trace("//new Object() | -0.0");
assert_bitor({}, -0.0);

trace("//\"0.0\" | -0.0");
assert_bitor("0.0", -0.0);

trace("//\"NaN\" | -0.0");
assert_bitor("NaN", -0.0);

trace("//\"-0.0\" | -0.0");
assert_bitor("-0.0", -0.0);

trace("//\"Infinity\" | -0.0");
assert_bitor("Infinity", -0.0);

trace("//\"1.0\" | -0.0");
assert_bitor("1.0", -0.0);

trace("//\"-1.0\" | -0.0");
assert_bitor("-1.0", -0.0);

trace("//\"0xFF1306\" | -0.0");
assert_bitor("0xFF1306", -0.0);

trace("//true | Infinity");
assert_bitor(true, Infinity);

trace("//false | Infinity");
assert_bitor(false, Infinity);

trace("//null | Infinity");
assert_bitor(null, Infinity);

trace("//undefined | Infinity");
assert_bitor(undefined, Infinity);

trace("//\"\" | Infinity");
assert_bitor("", Infinity);

trace("//\"str\" | Infinity");
assert_bitor("str", Infinity);

trace("//\"true\" | Infinity");
assert_bitor("true", Infinity);

trace("//\"false\" | Infinity");
assert_bitor("false", Infinity);

trace("//0.0 | Infinity");
assert_bitor(0.0, Infinity);

trace("//NaN | Infinity");
assert_bitor(NaN, Infinity);

trace("//-0.0 | Infinity");
assert_bitor(-0.0, Infinity);

trace("//Infinity | Infinity");
assert_bitor(Infinity, Infinity);

trace("//1.0 | Infinity");
assert_bitor(1.0, Infinity);

trace("//-1.0 | Infinity");
assert_bitor(-1.0, Infinity);

trace("//0xFF1306 | Infinity");
assert_bitor(0xFF1306, Infinity);

trace("//new Object() | Infinity");
assert_bitor({}, Infinity);

trace("//\"0.0\" | Infinity");
assert_bitor("0.0", Infinity);

trace("//\"NaN\" | Infinity");
assert_bitor("NaN", Infinity);

trace("//\"-0.0\" | Infinity");
assert_bitor("-0.0", Infinity);

trace("//\"Infinity\" | Infinity");
assert_bitor("Infinity", Infinity);

trace("//\"1.0\" | Infinity");
assert_bitor("1.0", Infinity);

trace("//\"-1.0\" | Infinity");
assert_bitor("-1.0", Infinity);

trace("//\"0xFF1306\" | Infinity");
assert_bitor("0xFF1306", Infinity);

trace("//true | 1.0");
assert_bitor(true, 1.0);

trace("//false | 1.0");
assert_bitor(false, 1.0);

trace("//null | 1.0");
assert_bitor(null, 1.0);

trace("//undefined | 1.0");
assert_bitor(undefined, 1.0);

trace("//\"\" | 1.0");
assert_bitor("", 1.0);

trace("//\"str\" | 1.0");
assert_bitor("str", 1.0);

trace("//\"true\" | 1.0");
assert_bitor("true", 1.0);

trace("//\"false\" | 1.0");
assert_bitor("false", 1.0);

trace("//0.0 | 1.0");
assert_bitor(0.0, 1.0);

trace("//NaN | 1.0");
assert_bitor(NaN, 1.0);

trace("//-0.0 | 1.0");
assert_bitor(-0.0, 1.0);

trace("//Infinity | 1.0");
assert_bitor(Infinity, 1.0);

trace("//1.0 | 1.0");
assert_bitor(1.0, 1.0);

trace("//-1.0 | 1.0");
assert_bitor(-1.0, 1.0);

trace("//0xFF1306 | 1.0");
assert_bitor(0xFF1306, 1.0);

trace("//new Object() | 1.0");
assert_bitor({}, 1.0);

trace("//\"0.0\" | 1.0");
assert_bitor("0.0", 1.0);

trace("//\"NaN\" | 1.0");
assert_bitor("NaN", 1.0);

trace("//\"-0.0\" | 1.0");
assert_bitor("-0.0", 1.0);

trace("//\"Infinity\" | 1.0");
assert_bitor("Infinity", 1.0);

trace("//\"1.0\" | 1.0");
assert_bitor("1.0", 1.0);

trace("//\"-1.0\" | 1.0");
assert_bitor("-1.0", 1.0);

trace("//\"0xFF1306\" | 1.0");
assert_bitor("0xFF1306", 1.0);

trace("//true | -1.0");
assert_bitor(true, -1.0);

trace("//false | -1.0");
assert_bitor(false, -1.0);

trace("//null | -1.0");
assert_bitor(null, -1.0);

trace("//undefined | -1.0");
assert_bitor(undefined, -1.0);

trace("//\"\" | -1.0");
assert_bitor("", -1.0);

trace("//\"str\" | -1.0");
assert_bitor("str", -1.0);

trace("//\"true\" | -1.0");
assert_bitor("true", -1.0);

trace("//\"false\" | -1.0");
assert_bitor("false", -1.0);

trace("//0.0 | -1.0");
assert_bitor(0.0, -1.0);

trace("//NaN | -1.0");
assert_bitor(NaN, -1.0);

trace("//-0.0 | -1.0");
assert_bitor(-0.0, -1.0);

trace("//Infinity | -1.0");
assert_bitor(Infinity, -1.0);

trace("//1.0 | -1.0");
assert_bitor(1.0, -1.0);

trace("//-1.0 | -1.0");
assert_bitor(-1.0, -1.0);

trace("//0xFF1306 | -1.0");
assert_bitor(0xFF1306, -1.0);

trace("//new Object() | -1.0");
assert_bitor({}, -1.0);

trace("//\"0.0\" | -1.0");
assert_bitor("0.0", -1.0);

trace("//\"NaN\" | -1.0");
assert_bitor("NaN", -1.0);

trace("//\"-0.0\" | -1.0");
assert_bitor("-0.0", -1.0);

trace("//\"Infinity\" | -1.0");
assert_bitor("Infinity", -1.0);

trace("//\"1.0\" | -1.0");
assert_bitor("1.0", -1.0);

trace("//\"-1.0\" | -1.0");
assert_bitor("-1.0", -1.0);

trace("//\"0xFF1306\" | -1.0");
assert_bitor("0xFF1306", -1.0);

trace("//true | 0xFF1306");
assert_bitor(true, 0xFF1306);

trace("//false | 0xFF1306");
assert_bitor(false, 0xFF1306);

trace("//null | 0xFF1306");
assert_bitor(null, 0xFF1306);

trace("//undefined | 0xFF1306");
assert_bitor(undefined, 0xFF1306);

trace("//\"\" | 0xFF1306");
assert_bitor("", 0xFF1306);

trace("//\"str\" | 0xFF1306");
assert_bitor("str", 0xFF1306);

trace("//\"true\" | 0xFF1306");
assert_bitor("true", 0xFF1306);

trace("//\"false\" | 0xFF1306");
assert_bitor("false", 0xFF1306);

trace("//0.0 | 0xFF1306");
assert_bitor(0.0, 0xFF1306);

trace("//NaN | 0xFF1306");
assert_bitor(NaN, 0xFF1306);

trace("//-0.0 | 0xFF1306");
assert_bitor(-0.0, 0xFF1306);

trace("//Infinity | 0xFF1306");
assert_bitor(Infinity, 0xFF1306);

trace("//1.0 | 0xFF1306");
assert_bitor(1.0, 0xFF1306);

trace("//-1.0 | 0xFF1306");
assert_bitor(-1.0, 0xFF1306);

trace("//0xFF1306 | 0xFF1306");
assert_bitor(0xFF1306, 0xFF1306);

trace("//new Object() | 0xFF1306");
assert_bitor({}, 0xFF1306);

trace("//\"0.0\" | 0xFF1306");
assert_bitor("0.0", 0xFF1306);

trace("//\"NaN\" | 0xFF1306");
assert_bitor("NaN", 0xFF1306);

trace("//\"-0.0\" | 0xFF1306");
assert_bitor("-0.0", 0xFF1306);

trace("//\"Infinity\" | 0xFF1306");
assert_bitor("Infinity", 0xFF1306);

trace("//\"1.0\" | 0xFF1306");
assert_bitor("1.0", 0xFF1306);

trace("//\"-1.0\" | 0xFF1306");
assert_bitor("-1.0", 0xFF1306);

trace("//\"0xFF1306\" | 0xFF1306");
assert_bitor("0xFF1306", 0xFF1306);

trace("//true | new Object()");
assert_bitor(true, {});

trace("//false | new Object()");
assert_bitor(false, {});

trace("//null | new Object()");
assert_bitor(null, {});

trace("//undefined | new Object()");
assert_bitor(undefined, {});

trace("//\"\" | new Object()");
assert_bitor("", {});

trace("//\"str\" | new Object()");
assert_bitor("str", {});

trace("//\"true\" | new Object()");
assert_bitor("true", {});

trace("//\"false\" | new Object()");
assert_bitor("false", {});

trace("//0.0 | new Object()");
assert_bitor(0.0, {});

trace("//NaN | new Object()");
assert_bitor(NaN, {});

trace("//-0.0 | new Object()");
assert_bitor(-0.0, {});

trace("//Infinity | new Object()");
assert_bitor(Infinity, {});

trace("//1.0 | new Object()");
assert_bitor(1.0, {});

trace("//-1.0 | new Object()");
assert_bitor(-1.0, {});

trace("//0xFF1306 | new Object()");
assert_bitor(0xFF1306, {});

trace("//new Object() | new Object()");
assert_bitor({}, {});

trace("//\"0.0\" | new Object()");
assert_bitor("0.0", {});

trace("//\"NaN\" | new Object()");
assert_bitor("NaN", {});

trace("//\"-0.0\" | new Object()");
assert_bitor("-0.0", {});

trace("//\"Infinity\" | new Object()");
assert_bitor("Infinity", {});

trace("//\"1.0\" | new Object()");
assert_bitor("1.0", {});

trace("//\"-1.0\" | new Object()");
assert_bitor("-1.0", {});

trace("//\"0xFF1306\" | new Object()");
assert_bitor("0xFF1306", {});

trace("//true | \"0.0\"");
assert_bitor(true, "0.0");

trace("//false | \"0.0\"");
assert_bitor(false, "0.0");

trace("//null | \"0.0\"");
assert_bitor(null, "0.0");

trace("//undefined | \"0.0\"");
assert_bitor(undefined, "0.0");

trace("//\"\" | \"0.0\"");
assert_bitor("", "0.0");

trace("//\"str\" | \"0.0\"");
assert_bitor("str", "0.0");

trace("//\"true\" | \"0.0\"");
assert_bitor("true", "0.0");

trace("//\"false\" | \"0.0\"");
assert_bitor("false", "0.0");

trace("//0.0 | \"0.0\"");
assert_bitor(0.0, "0.0");

trace("//NaN | \"0.0\"");
assert_bitor(NaN, "0.0");

trace("//-0.0 | \"0.0\"");
assert_bitor(-0.0, "0.0");

trace("//Infinity | \"0.0\"");
assert_bitor(Infinity, "0.0");

trace("//1.0 | \"0.0\"");
assert_bitor(1.0, "0.0");

trace("//-1.0 | \"0.0\"");
assert_bitor(-1.0, "0.0");

trace("//0xFF1306 | \"0.0\"");
assert_bitor(0xFF1306, "0.0");

trace("//new Object() | \"0.0\"");
assert_bitor({}, "0.0");

trace("//\"0.0\" | \"0.0\"");
assert_bitor("0.0", "0.0");

trace("//\"NaN\" | \"0.0\"");
assert_bitor("NaN", "0.0");

trace("//\"-0.0\" | \"0.0\"");
assert_bitor("-0.0", "0.0");

trace("//\"Infinity\" | \"0.0\"");
assert_bitor("Infinity", "0.0");

trace("//\"1.0\" | \"0.0\"");
assert_bitor("1.0", "0.0");

trace("//\"-1.0\" | \"0.0\"");
assert_bitor("-1.0", "0.0");

trace("//\"0xFF1306\" | \"0.0\"");
assert_bitor("0xFF1306", "0.0");

trace("//true | \"NaN\"");
assert_bitor(true, "NaN");

trace("//false | \"NaN\"");
assert_bitor(false, "NaN");

trace("//null | \"NaN\"");
assert_bitor(null, "NaN");

trace("//undefined | \"NaN\"");
assert_bitor(undefined, "NaN");

trace("//\"\" | \"NaN\"");
assert_bitor("", "NaN");

trace("//\"str\" | \"NaN\"");
assert_bitor("str", "NaN");

trace("//\"true\" | \"NaN\"");
assert_bitor("true", "NaN");

trace("//\"false\" | \"NaN\"");
assert_bitor("false", "NaN");

trace("//0.0 | \"NaN\"");
assert_bitor(0.0, "NaN");

trace("//NaN | \"NaN\"");
assert_bitor(NaN, "NaN");

trace("//-0.0 | \"NaN\"");
assert_bitor(-0.0, "NaN");

trace("//Infinity | \"NaN\"");
assert_bitor(Infinity, "NaN");

trace("//1.0 | \"NaN\"");
assert_bitor(1.0, "NaN");

trace("//-1.0 | \"NaN\"");
assert_bitor(-1.0, "NaN");

trace("//0xFF1306 | \"NaN\"");
assert_bitor(0xFF1306, "NaN");

trace("//new Object() | \"NaN\"");
assert_bitor({}, "NaN");

trace("//\"0.0\" | \"NaN\"");
assert_bitor("0.0", "NaN");

trace("//\"NaN\" | \"NaN\"");
assert_bitor("NaN", "NaN");

trace("//\"-0.0\" | \"NaN\"");
assert_bitor("-0.0", "NaN");

trace("//\"Infinity\" | \"NaN\"");
assert_bitor("Infinity", "NaN");

trace("//\"1.0\" | \"NaN\"");
assert_bitor("1.0", "NaN");

trace("//\"-1.0\" | \"NaN\"");
assert_bitor("-1.0", "NaN");

trace("//\"0xFF1306\" | \"NaN\"");
assert_bitor("0xFF1306", "NaN");

trace("//true | \"-0.0\"");
assert_bitor(true, "-0.0");

trace("//false | \"-0.0\"");
assert_bitor(false, "-0.0");

trace("//null | \"-0.0\"");
assert_bitor(null, "-0.0");

trace("//undefined | \"-0.0\"");
assert_bitor(undefined, "-0.0");

trace("//\"\" | \"-0.0\"");
assert_bitor("", "-0.0");

trace("//\"str\" | \"-0.0\"");
assert_bitor("str", "-0.0");

trace("//\"true\" | \"-0.0\"");
assert_bitor("true", "-0.0");

trace("//\"false\" | \"-0.0\"");
assert_bitor("false", "-0.0");

trace("//0.0 | \"-0.0\"");
assert_bitor(0.0, "-0.0");

trace("//NaN | \"-0.0\"");
assert_bitor(NaN, "-0.0");

trace("//-0.0 | \"-0.0\"");
assert_bitor(-0.0, "-0.0");

trace("//Infinity | \"-0.0\"");
assert_bitor(Infinity, "-0.0");

trace("//1.0 | \"-0.0\"");
assert_bitor(1.0, "-0.0");

trace("//-1.0 | \"-0.0\"");
assert_bitor(-1.0, "-0.0");

trace("//0xFF1306 | \"-0.0\"");
assert_bitor(0xFF1306, "-0.0");

trace("//new Object() | \"-0.0\"");
assert_bitor({}, "-0.0");

trace("//\"0.0\" | \"-0.0\"");
assert_bitor("0.0", "-0.0");

trace("//\"NaN\" | \"-0.0\"");
assert_bitor("NaN", "-0.0");

trace("//\"-0.0\" | \"-0.0\"");
assert_bitor("-0.0", "-0.0");

trace("//\"Infinity\" | \"-0.0\"");
assert_bitor("Infinity", "-0.0");

trace("//\"1.0\" | \"-0.0\"");
assert_bitor("1.0", "-0.0");

trace("//\"-1.0\" | \"-0.0\"");
assert_bitor("-1.0", "-0.0");

trace("//\"0xFF1306\" | \"-0.0\"");
assert_bitor("0xFF1306", "-0.0");

trace("//true | \"Infinity\"");
assert_bitor(true, "Infinity");

trace("//false | \"Infinity\"");
assert_bitor(false, "Infinity");

trace("//null | \"Infinity\"");
assert_bitor(null, "Infinity");

trace("//undefined | \"Infinity\"");
assert_bitor(undefined, "Infinity");

trace("//\"\" | \"Infinity\"");
assert_bitor("", "Infinity");

trace("//\"str\" | \"Infinity\"");
assert_bitor("str", "Infinity");

trace("//\"true\" | \"Infinity\"");
assert_bitor("true", "Infinity");

trace("//\"false\" | \"Infinity\"");
assert_bitor("false", "Infinity");

trace("//0.0 | \"Infinity\"");
assert_bitor(0.0, "Infinity");

trace("//NaN | \"Infinity\"");
assert_bitor(NaN, "Infinity");

trace("//-0.0 | \"Infinity\"");
assert_bitor(-0.0, "Infinity");

trace("//Infinity | \"Infinity\"");
assert_bitor(Infinity, "Infinity");

trace("//1.0 | \"Infinity\"");
assert_bitor(1.0, "Infinity");

trace("//-1.0 | \"Infinity\"");
assert_bitor(-1.0, "Infinity");

trace("//0xFF1306 | \"Infinity\"");
assert_bitor(0xFF1306, "Infinity");

trace("//new Object() | \"Infinity\"");
assert_bitor({}, "Infinity");

trace("//\"0.0\" | \"Infinity\"");
assert_bitor("0.0", "Infinity");

trace("//\"NaN\" | \"Infinity\"");
assert_bitor("NaN", "Infinity");

trace("//\"-0.0\" | \"Infinity\"");
assert_bitor("-0.0", "Infinity");

trace("//\"Infinity\" | \"Infinity\"");
assert_bitor("Infinity", "Infinity");

trace("//\"1.0\" | \"Infinity\"");
assert_bitor("1.0", "Infinity");

trace("//\"-1.0\" | \"Infinity\"");
assert_bitor("-1.0", "Infinity");

trace("//\"0xFF1306\" | \"Infinity\"");
assert_bitor("0xFF1306", "Infinity");

trace("//true | \"1.0\"");
assert_bitor(true, "1.0");

trace("//false | \"1.0\"");
assert_bitor(false, "1.0");

trace("//null | \"1.0\"");
assert_bitor(null, "1.0");

trace("//undefined | \"1.0\"");
assert_bitor(undefined, "1.0");

trace("//\"\" | \"1.0\"");
assert_bitor("", "1.0");

trace("//\"str\" | \"1.0\"");
assert_bitor("str", "1.0");

trace("//\"true\" | \"1.0\"");
assert_bitor("true", "1.0");

trace("//\"false\" | \"1.0\"");
assert_bitor("false", "1.0");

trace("//0.0 | \"1.0\"");
assert_bitor(0.0, "1.0");

trace("//NaN | \"1.0\"");
assert_bitor(NaN, "1.0");

trace("//-0.0 | \"1.0\"");
assert_bitor(-0.0, "1.0");

trace("//Infinity | \"1.0\"");
assert_bitor(Infinity, "1.0");

trace("//1.0 | \"1.0\"");
assert_bitor(1.0, "1.0");

trace("//-1.0 | \"1.0\"");
assert_bitor(-1.0, "1.0");

trace("//0xFF1306 | \"1.0\"");
assert_bitor(0xFF1306, "1.0");

trace("//new Object() | \"1.0\"");
assert_bitor({}, "1.0");

trace("//\"0.0\" | \"1.0\"");
assert_bitor("0.0", "1.0");

trace("//\"NaN\" | \"1.0\"");
assert_bitor("NaN", "1.0");

trace("//\"-0.0\" | \"1.0\"");
assert_bitor("-0.0", "1.0");

trace("//\"Infinity\" | \"1.0\"");
assert_bitor("Infinity", "1.0");

trace("//\"1.0\" | \"1.0\"");
assert_bitor("1.0", "1.0");

trace("//\"-1.0\" | \"1.0\"");
assert_bitor("-1.0", "1.0");

trace("//\"0xFF1306\" | \"1.0\"");
assert_bitor("0xFF1306", "1.0");

trace("//true | \"-1.0\"");
assert_bitor(true, "-1.0");

trace("//false | \"-1.0\"");
assert_bitor(false, "-1.0");

trace("//null | \"-1.0\"");
assert_bitor(null, "-1.0");

trace("//undefined | \"-1.0\"");
assert_bitor(undefined, "-1.0");

trace("//\"\" | \"-1.0\"");
assert_bitor("", "-1.0");

trace("//\"str\" | \"-1.0\"");
assert_bitor("str", "-1.0");

trace("//\"true\" | \"-1.0\"");
assert_bitor("true", "-1.0");

trace("//\"false\" | \"-1.0\"");
assert_bitor("false", "-1.0");

trace("//0.0 | \"-1.0\"");
assert_bitor(0.0, "-1.0");

trace("//NaN | \"-1.0\"");
assert_bitor(NaN, "-1.0");

trace("//-0.0 | \"-1.0\"");
assert_bitor(-0.0, "-1.0");

trace("//Infinity | \"-1.0\"");
assert_bitor(Infinity, "-1.0");

trace("//1.0 | \"-1.0\"");
assert_bitor(1.0, "-1.0");

trace("//-1.0 | \"-1.0\"");
assert_bitor(-1.0, "-1.0");

trace("//0xFF1306 | \"-1.0\"");
assert_bitor(0xFF1306, "-1.0");

trace("//new Object() | \"-1.0\"");
assert_bitor({}, "-1.0");

trace("//\"0.0\" | \"-1.0\"");
assert_bitor("0.0", "-1.0");

trace("//\"NaN\" | \"-1.0\"");
assert_bitor("NaN", "-1.0");

trace("//\"-0.0\" | \"-1.0\"");
assert_bitor("-0.0", "-1.0");

trace("//\"Infinity\" | \"-1.0\"");
assert_bitor("Infinity", "-1.0");

trace("//\"1.0\" | \"-1.0\"");
assert_bitor("1.0", "-1.0");

trace("//\"-1.0\" | \"-1.0\"");
assert_bitor("-1.0", "-1.0");

trace("//\"0xFF1306\" | \"-1.0\"");
assert_bitor("0xFF1306", "-1.0");

trace("//true | \"0xFF1306\"");
assert_bitor(true, "0xFF1306");

trace("//false | \"0xFF1306\"");
assert_bitor(false, "0xFF1306");

trace("//null | \"0xFF1306\"");
assert_bitor(null, "0xFF1306");

trace("//undefined | \"0xFF1306\"");
assert_bitor(undefined, "0xFF1306");

trace("//\"\" | \"0xFF1306\"");
assert_bitor("", "0xFF1306");

trace("//\"str\" | \"0xFF1306\"");
assert_bitor("str", "0xFF1306");

trace("//\"true\" | \"0xFF1306\"");
assert_bitor("true", "0xFF1306");

trace("//\"false\" | \"0xFF1306\"");
assert_bitor("false", "0xFF1306");

trace("//0.0 | \"0xFF1306\"");
assert_bitor(0.0, "0xFF1306");

trace("//NaN | \"0xFF1306\"");
assert_bitor(NaN, "0xFF1306");

trace("//-0.0 | \"0xFF1306\"");
assert_bitor(-0.0, "0xFF1306");

trace("//Infinity | \"0xFF1306\"");
assert_bitor(Infinity, "0xFF1306");

trace("//1.0 | \"0xFF1306\"");
assert_bitor(1.0, "0xFF1306");

trace("//-1.0 | \"0xFF1306\"");
assert_bitor(-1.0, "0xFF1306");

trace("//0xFF1306 | \"0xFF1306\"");
assert_bitor(0xFF1306, "0xFF1306");

trace("//new Object() | \"0xFF1306\"");
assert_bitor({}, "0xFF1306");

trace("//\"0.0\" | \"0xFF1306\"");
assert_bitor("0.0", "0xFF1306");

trace("//\"NaN\" | \"0xFF1306\"");
assert_bitor("NaN", "0xFF1306");

trace("//\"-0.0\" | \"0xFF1306\"");
assert_bitor("-0.0", "0xFF1306");

trace("//\"Infinity\" | \"0xFF1306\"");
assert_bitor("Infinity", "0xFF1306");

trace("//\"1.0\" | \"0xFF1306\"");
assert_bitor("1.0", "0xFF1306");

trace("//\"-1.0\" | \"0xFF1306\"");
assert_bitor("-1.0", "0xFF1306");

trace("//\"0xFF1306\" | \"0xFF1306\"");
assert_bitor("0xFF1306", "0xFF1306");