package {
	public class Test {
	}
}

function assert_subtract(val1, val2) {
	trace(val1 - val2);
}

trace("//true - true");
assert_subtract(true, true);

trace("//false - true");
assert_subtract(false, true);

trace("//null - true");
assert_subtract(null, true);

trace("//undefined - true");
assert_subtract(undefined, true);

trace("//\"\" - true");
assert_subtract("", true);

trace("//\"str\" - true");
assert_subtract("str", true);

trace("//\"true\" - true");
assert_subtract("true", true);

trace("//\"false\" - true");
assert_subtract("false", true);

trace("//0.0 - true");
assert_subtract(0.0, true);

trace("//NaN - true");
assert_subtract(NaN, true);

trace("//-0.0 - true");
assert_subtract(-0.0, true);

trace("//Infinity - true");
assert_subtract(Infinity, true);

trace("//1.0 - true");
assert_subtract(1.0, true);

trace("//-1.0 - true");
assert_subtract(-1.0, true);

trace("//0xFF1306 - true");
assert_subtract(0xFF1306, true);

trace("//new Object() - true");
assert_subtract({}, true);

trace("//\"0.0\" - true");
assert_subtract("0.0", true);

trace("//\"NaN\" - true");
assert_subtract("NaN", true);

trace("//\"-0.0\" - true");
assert_subtract("-0.0", true);

trace("//\"Infinity\" - true");
assert_subtract("Infinity", true);

trace("//\"1.0\" - true");
assert_subtract("1.0", true);

trace("//\"-1.0\" - true");
assert_subtract("-1.0", true);

trace("//\"0xFF1306\" - true");
assert_subtract("0xFF1306", true);

trace("//true - false");
assert_subtract(true, false);

trace("//false - false");
assert_subtract(false, false);

trace("//null - false");
assert_subtract(null, false);

trace("//undefined - false");
assert_subtract(undefined, false);

trace("//\"\" - false");
assert_subtract("", false);

trace("//\"str\" - false");
assert_subtract("str", false);

trace("//\"true\" - false");
assert_subtract("true", false);

trace("//\"false\" - false");
assert_subtract("false", false);

trace("//0.0 - false");
assert_subtract(0.0, false);

trace("//NaN - false");
assert_subtract(NaN, false);

trace("//-0.0 - false");
assert_subtract(-0.0, false);

trace("//Infinity - false");
assert_subtract(Infinity, false);

trace("//1.0 - false");
assert_subtract(1.0, false);

trace("//-1.0 - false");
assert_subtract(-1.0, false);

trace("//0xFF1306 - false");
assert_subtract(0xFF1306, false);

trace("//new Object() - false");
assert_subtract({}, false);

trace("//\"0.0\" - false");
assert_subtract("0.0", false);

trace("//\"NaN\" - false");
assert_subtract("NaN", false);

trace("//\"-0.0\" - false");
assert_subtract("-0.0", false);

trace("//\"Infinity\" - false");
assert_subtract("Infinity", false);

trace("//\"1.0\" - false");
assert_subtract("1.0", false);

trace("//\"-1.0\" - false");
assert_subtract("-1.0", false);

trace("//\"0xFF1306\" - false");
assert_subtract("0xFF1306", false);
trace("//true - null");
assert_subtract(true, null);

trace("//false - null");
assert_subtract(false, null);

trace("//null - null");
assert_subtract(null, null);

trace("//undefined - null");
assert_subtract(undefined, null);

trace("//\"\" - null");
assert_subtract("", null);

trace("//\"str\" - null");
assert_subtract("str", null);

trace("//\"true\" - null");
assert_subtract("true", null);

trace("//\"false\" - null");
assert_subtract("false", null);

trace("//0.0 - null");
assert_subtract(0.0, null);

trace("//NaN - null");
assert_subtract(NaN, null);

trace("//-0.0 - null");
assert_subtract(-0.0, null);

trace("//Infinity - null");
assert_subtract(Infinity, null);

trace("//1.0 - null");
assert_subtract(1.0, null);

trace("//-1.0 - null");
assert_subtract(-1.0, null);

trace("//0xFF1306 - null");
assert_subtract(0xFF1306, null);

trace("//new Object() - null");
assert_subtract({}, null);

trace("//\"0.0\" - null");
assert_subtract("0.0", null);

trace("//\"NaN\" - null");
assert_subtract("NaN", null);

trace("//\"-0.0\" - null");
assert_subtract("-0.0", null);

trace("//\"Infinity\" - null");
assert_subtract("Infinity", null);

trace("//\"1.0\" - null");
assert_subtract("1.0", null);

trace("//\"-1.0\" - null");
assert_subtract("-1.0", null);

trace("//\"0xFF1306\" - null");
assert_subtract("0xFF1306", null);

trace("//true - undefined");
assert_subtract(true, undefined);

trace("//false - undefined");
assert_subtract(false, undefined);

trace("//null - undefined");
assert_subtract(null, undefined);

trace("//undefined - undefined");
assert_subtract(undefined, undefined);

trace("//\"\" - undefined");
assert_subtract("", undefined);

trace("//\"str\" - undefined");
assert_subtract("str", undefined);

trace("//\"true\" - undefined");
assert_subtract("true", undefined);

trace("//\"false\" - undefined");
assert_subtract("false", undefined);

trace("//0.0 - undefined");
assert_subtract(0.0, undefined);

trace("//NaN - undefined");
assert_subtract(NaN, undefined);

trace("//-0.0 - undefined");
assert_subtract(-0.0, undefined);

trace("//Infinity - undefined");
assert_subtract(Infinity, undefined);

trace("//1.0 - undefined");
assert_subtract(1.0, undefined);

trace("//-1.0 - undefined");
assert_subtract(-1.0, undefined);

trace("//0xFF1306 - undefined");
assert_subtract(0xFF1306, undefined);

trace("//new Object() - undefined");
assert_subtract({}, undefined);

trace("//\"0.0\" - undefined");
assert_subtract("0.0", undefined);

trace("//\"NaN\" - undefined");
assert_subtract("NaN", undefined);

trace("//\"-0.0\" - undefined");
assert_subtract("-0.0", undefined);

trace("//\"Infinity\" - undefined");
assert_subtract("Infinity", undefined);

trace("//\"1.0\" - undefined");
assert_subtract("1.0", undefined);

trace("//\"-1.0\" - undefined");
assert_subtract("-1.0", undefined);

trace("//\"0xFF1306\" - undefined");
assert_subtract("0xFF1306", undefined);

trace("//true - \"\"");
assert_subtract(true, "");

trace("//false - \"\"");
assert_subtract(false, "");

trace("//null - \"\"");
assert_subtract(null, "");

trace("//undefined - \"\"");
assert_subtract(undefined, "");

trace("//\"\" - \"\"");
assert_subtract("", "");

trace("//\"str\" - \"\"");
assert_subtract("str", "");

trace("//\"true\" - \"\"");
assert_subtract("true", "");

trace("//\"false\" - \"\"");
assert_subtract("false", "");

trace("//0.0 - \"\"");
assert_subtract(0.0, "");

trace("//NaN - \"\"");
assert_subtract(NaN, "");

trace("//-0.0 - \"\"");
assert_subtract(-0.0, "");

trace("//Infinity - \"\"");
assert_subtract(Infinity, "");

trace("//1.0 - \"\"");
assert_subtract(1.0, "");

trace("//-1.0 - \"\"");
assert_subtract(-1.0, "");

trace("//0xFF1306 - \"\"");
assert_subtract(0xFF1306, "");

trace("//new Object() - \"\"");
assert_subtract({}, "");

trace("//\"0.0\" - \"\"");
assert_subtract("0.0", "");

trace("//\"NaN\" - \"\"");
assert_subtract("NaN", "");

trace("//\"-0.0\" - \"\"");
assert_subtract("-0.0", "");

trace("//\"Infinity\" - \"\"");
assert_subtract("Infinity", "");

trace("//\"1.0\" - \"\"");
assert_subtract("1.0", "");

trace("//\"-1.0\" - \"\"");
assert_subtract("-1.0", "");

trace("//\"0xFF1306\" - \"\"");
assert_subtract("0xFF1306", "");

trace("//true - \"str\"");
assert_subtract(true, "str");

trace("//false - \"str\"");
assert_subtract(false, "str");

trace("//null - \"str\"");
assert_subtract(null, "str");

trace("//undefined - \"str\"");
assert_subtract(undefined, "str");

trace("//\"\" - \"str\"");
assert_subtract("", "str");

trace("//\"str\" - \"str\"");
assert_subtract("str", "str");

trace("//\"true\" - \"str\"");
assert_subtract("true", "str");

trace("//\"false\" - \"str\"");
assert_subtract("false", "str");

trace("//0.0 - \"str\"");
assert_subtract(0.0, "str");

trace("//NaN - \"str\"");
assert_subtract(NaN, "str");

trace("//-0.0 - \"str\"");
assert_subtract(-0.0, "str");

trace("//Infinity - \"str\"");
assert_subtract(Infinity, "str");

trace("//1.0 - \"str\"");
assert_subtract(1.0, "str");

trace("//-1.0 - \"str\"");
assert_subtract(-1.0, "str");

trace("//0xFF1306 - \"str\"");
assert_subtract(0xFF1306, "str");

trace("//new Object() - \"str\"");
assert_subtract({}, "str");

trace("//\"0.0\" - \"str\"");
assert_subtract("0.0", "str");

trace("//\"NaN\" - \"str\"");
assert_subtract("NaN", "str");

trace("//\"-0.0\" - \"str\"");
assert_subtract("-0.0", "str");

trace("//\"Infinity\" - \"str\"");
assert_subtract("Infinity", "str");

trace("//\"1.0\" - \"str\"");
assert_subtract("1.0", "str");

trace("//\"-1.0\" - \"str\"");
assert_subtract("-1.0", "str");

trace("//\"0xFF1306\" - \"str\"");
assert_subtract("0xFF1306", "str");

trace("//true - \"true\"");
assert_subtract(true, "true");

trace("//false - \"true\"");
assert_subtract(false, "true");

trace("//null - \"true\"");
assert_subtract(null, "true");

trace("//undefined - \"true\"");
assert_subtract(undefined, "true");

trace("//\"\" - \"true\"");
assert_subtract("", "true");

trace("//\"str\" - \"true\"");
assert_subtract("str", "true");

trace("//\"true\" - \"true\"");
assert_subtract("true", "true");

trace("//\"false\" - \"true\"");
assert_subtract("false", "true");

trace("//0.0 - \"true\"");
assert_subtract(0.0, "true");

trace("//NaN - \"true\"");
assert_subtract(NaN, "true");

trace("//-0.0 - \"true\"");
assert_subtract(-0.0, "true");

trace("//Infinity - \"true\"");
assert_subtract(Infinity, "true");

trace("//1.0 - \"true\"");
assert_subtract(1.0, "true");

trace("//-1.0 - \"true\"");
assert_subtract(-1.0, "true");

trace("//0xFF1306 - \"true\"");
assert_subtract(0xFF1306, "true");

trace("//new Object() - \"true\"");
assert_subtract({}, "true");

trace("//\"0.0\" - \"true\"");
assert_subtract("0.0", "true");

trace("//\"NaN\" - \"true\"");
assert_subtract("NaN", "true");

trace("//\"-0.0\" - \"true\"");
assert_subtract("-0.0", "true");

trace("//\"Infinity\" - \"true\"");
assert_subtract("Infinity", "true");

trace("//\"1.0\" - \"true\"");
assert_subtract("1.0", "true");

trace("//\"-1.0\" - \"true\"");
assert_subtract("-1.0", "true");

trace("//\"0xFF1306\" - \"true\"");
assert_subtract("0xFF1306", "true");

trace("//true - \"false\"");
assert_subtract(true, "false");

trace("//false - \"false\"");
assert_subtract(false, "false");

trace("//null - \"false\"");
assert_subtract(null, "false");

trace("//undefined - \"false\"");
assert_subtract(undefined, "false");

trace("//\"\" - \"false\"");
assert_subtract("", "false");

trace("//\"str\" - \"false\"");
assert_subtract("str", "false");

trace("//\"true\" - \"false\"");
assert_subtract("true", "false");

trace("//\"false\" - \"false\"");
assert_subtract("false", "false");

trace("//0.0 - \"false\"");
assert_subtract(0.0, "false");

trace("//NaN - \"false\"");
assert_subtract(NaN, "false");

trace("//-0.0 - \"false\"");
assert_subtract(-0.0, "false");

trace("//Infinity - \"false\"");
assert_subtract(Infinity, "false");

trace("//1.0 - \"false\"");
assert_subtract(1.0, "false");

trace("//-1.0 - \"false\"");
assert_subtract(-1.0, "false");

trace("//0xFF1306 - \"false\"");
assert_subtract(0xFF1306, "false");

trace("//new Object() - \"false\"");
assert_subtract({}, "false");

trace("//\"0.0\" - \"false\"");
assert_subtract("0.0", "false");

trace("//\"NaN\" - \"false\"");
assert_subtract("NaN", "false");

trace("//\"-0.0\" - \"false\"");
assert_subtract("-0.0", "false");

trace("//\"Infinity\" - \"false\"");
assert_subtract("Infinity", "false");

trace("//\"1.0\" - \"false\"");
assert_subtract("1.0", "false");

trace("//\"-1.0\" - \"false\"");
assert_subtract("-1.0", "false");

trace("//\"0xFF1306\" - \"false\"");
assert_subtract("0xFF1306", "false");

trace("//true - 0.0");
assert_subtract(true, 0.0);

trace("//false - 0.0");
assert_subtract(false, 0.0);

trace("//null - 0.0");
assert_subtract(null, 0.0);

trace("//undefined - 0.0");
assert_subtract(undefined, 0.0);

trace("//\"\" - 0.0");
assert_subtract("", 0.0);

trace("//\"str\" - 0.0");
assert_subtract("str", 0.0);

trace("//\"true\" - 0.0");
assert_subtract("true", 0.0);

trace("//\"false\" - 0.0");
assert_subtract("false", 0.0);

trace("//0.0 - 0.0");
assert_subtract(0.0, 0.0);

trace("//NaN - 0.0");
assert_subtract(NaN, 0.0);

trace("//-0.0 - 0.0");
assert_subtract(-0.0, 0.0);

trace("//Infinity - 0.0");
assert_subtract(Infinity, 0.0);

trace("//1.0 - 0.0");
assert_subtract(1.0, 0.0);

trace("//-1.0 - 0.0");
assert_subtract(-1.0, 0.0);

trace("//0xFF1306 - 0.0");
assert_subtract(0xFF1306, 0.0);

trace("//new Object() - 0.0");
assert_subtract({}, 0.0);

trace("//\"0.0\" - 0.0");
assert_subtract("0.0", 0.0);

trace("//\"NaN\" - 0.0");
assert_subtract("NaN", 0.0);

trace("//\"-0.0\" - 0.0");
assert_subtract("-0.0", 0.0);

trace("//\"Infinity\" - 0.0");
assert_subtract("Infinity", 0.0);

trace("//\"1.0\" - 0.0");
assert_subtract("1.0", 0.0);

trace("//\"-1.0\" - 0.0");
assert_subtract("-1.0", 0.0);

trace("//\"0xFF1306\" - 0.0");
assert_subtract("0xFF1306", 0.0);

trace("//true - NaN");
assert_subtract(true, NaN);

trace("//false - NaN");
assert_subtract(false, NaN);

trace("//null - NaN");
assert_subtract(null, NaN);

trace("//undefined - NaN");
assert_subtract(undefined, NaN);

trace("//\"\" - NaN");
assert_subtract("", NaN);

trace("//\"str\" - NaN");
assert_subtract("str", NaN);

trace("//\"true\" - NaN");
assert_subtract("true", NaN);

trace("//\"false\" - NaN");
assert_subtract("false", NaN);

trace("//0.0 - NaN");
assert_subtract(0.0, NaN);

trace("//NaN - NaN");
assert_subtract(NaN, NaN);

trace("//-0.0 - NaN");
assert_subtract(-0.0, NaN);

trace("//Infinity - NaN");
assert_subtract(Infinity, NaN);

trace("//1.0 - NaN");
assert_subtract(1.0, NaN);

trace("//-1.0 - NaN");
assert_subtract(-1.0, NaN);

trace("//0xFF1306 - NaN");
assert_subtract(0xFF1306, NaN);

trace("//new Object() - NaN");
assert_subtract({}, NaN);

trace("//\"0.0\" - NaN");
assert_subtract("0.0", NaN);

trace("//\"NaN\" - NaN");
assert_subtract("NaN", NaN);

trace("//\"-0.0\" - NaN");
assert_subtract("-0.0", NaN);

trace("//\"Infinity\" - NaN");
assert_subtract("Infinity", NaN);

trace("//\"1.0\" - NaN");
assert_subtract("1.0", NaN);

trace("//\"-1.0\" - NaN");
assert_subtract("-1.0", NaN);

trace("//\"0xFF1306\" - NaN");
assert_subtract("0xFF1306", NaN);

trace("//true - -0.0");
assert_subtract(true, -0.0);

trace("//false - -0.0");
assert_subtract(false, -0.0);

trace("//null - -0.0");
assert_subtract(null, -0.0);

trace("//undefined - -0.0");
assert_subtract(undefined, -0.0);

trace("//\"\" - -0.0");
assert_subtract("", -0.0);

trace("//\"str\" - -0.0");
assert_subtract("str", -0.0);

trace("//\"true\" - -0.0");
assert_subtract("true", -0.0);

trace("//\"false\" - -0.0");
assert_subtract("false", -0.0);

trace("//0.0 - -0.0");
assert_subtract(0.0, -0.0);

trace("//NaN - -0.0");
assert_subtract(NaN, -0.0);

trace("//-0.0 - -0.0");
assert_subtract(-0.0, -0.0);

trace("//Infinity - -0.0");
assert_subtract(Infinity, -0.0);

trace("//1.0 - -0.0");
assert_subtract(1.0, -0.0);

trace("//-1.0 - -0.0");
assert_subtract(-1.0, -0.0);

trace("//0xFF1306 - -0.0");
assert_subtract(0xFF1306, -0.0);

trace("//new Object() - -0.0");
assert_subtract({}, -0.0);

trace("//\"0.0\" - -0.0");
assert_subtract("0.0", -0.0);

trace("//\"NaN\" - -0.0");
assert_subtract("NaN", -0.0);

trace("//\"-0.0\" - -0.0");
assert_subtract("-0.0", -0.0);

trace("//\"Infinity\" - -0.0");
assert_subtract("Infinity", -0.0);

trace("//\"1.0\" - -0.0");
assert_subtract("1.0", -0.0);

trace("//\"-1.0\" - -0.0");
assert_subtract("-1.0", -0.0);

trace("//\"0xFF1306\" - -0.0");
assert_subtract("0xFF1306", -0.0);

trace("//true - Infinity");
assert_subtract(true, Infinity);

trace("//false - Infinity");
assert_subtract(false, Infinity);

trace("//null - Infinity");
assert_subtract(null, Infinity);

trace("//undefined - Infinity");
assert_subtract(undefined, Infinity);

trace("//\"\" - Infinity");
assert_subtract("", Infinity);

trace("//\"str\" - Infinity");
assert_subtract("str", Infinity);

trace("//\"true\" - Infinity");
assert_subtract("true", Infinity);

trace("//\"false\" - Infinity");
assert_subtract("false", Infinity);

trace("//0.0 - Infinity");
assert_subtract(0.0, Infinity);

trace("//NaN - Infinity");
assert_subtract(NaN, Infinity);

trace("//-0.0 - Infinity");
assert_subtract(-0.0, Infinity);

trace("//Infinity - Infinity");
assert_subtract(Infinity, Infinity);

trace("//1.0 - Infinity");
assert_subtract(1.0, Infinity);

trace("//-1.0 - Infinity");
assert_subtract(-1.0, Infinity);

trace("//0xFF1306 - Infinity");
assert_subtract(0xFF1306, Infinity);

trace("//new Object() - Infinity");
assert_subtract({}, Infinity);

trace("//\"0.0\" - Infinity");
assert_subtract("0.0", Infinity);

trace("//\"NaN\" - Infinity");
assert_subtract("NaN", Infinity);

trace("//\"-0.0\" - Infinity");
assert_subtract("-0.0", Infinity);

trace("//\"Infinity\" - Infinity");
assert_subtract("Infinity", Infinity);

trace("//\"1.0\" - Infinity");
assert_subtract("1.0", Infinity);

trace("//\"-1.0\" - Infinity");
assert_subtract("-1.0", Infinity);

trace("//\"0xFF1306\" - Infinity");
assert_subtract("0xFF1306", Infinity);

trace("//true - 1.0");
assert_subtract(true, 1.0);

trace("//false - 1.0");
assert_subtract(false, 1.0);

trace("//null - 1.0");
assert_subtract(null, 1.0);

trace("//undefined - 1.0");
assert_subtract(undefined, 1.0);

trace("//\"\" - 1.0");
assert_subtract("", 1.0);

trace("//\"str\" - 1.0");
assert_subtract("str", 1.0);

trace("//\"true\" - 1.0");
assert_subtract("true", 1.0);

trace("//\"false\" - 1.0");
assert_subtract("false", 1.0);

trace("//0.0 - 1.0");
assert_subtract(0.0, 1.0);

trace("//NaN - 1.0");
assert_subtract(NaN, 1.0);

trace("//-0.0 - 1.0");
assert_subtract(-0.0, 1.0);

trace("//Infinity - 1.0");
assert_subtract(Infinity, 1.0);

trace("//1.0 - 1.0");
assert_subtract(1.0, 1.0);

trace("//-1.0 - 1.0");
assert_subtract(-1.0, 1.0);

trace("//0xFF1306 - 1.0");
assert_subtract(0xFF1306, 1.0);

trace("//new Object() - 1.0");
assert_subtract({}, 1.0);

trace("//\"0.0\" - 1.0");
assert_subtract("0.0", 1.0);

trace("//\"NaN\" - 1.0");
assert_subtract("NaN", 1.0);

trace("//\"-0.0\" - 1.0");
assert_subtract("-0.0", 1.0);

trace("//\"Infinity\" - 1.0");
assert_subtract("Infinity", 1.0);

trace("//\"1.0\" - 1.0");
assert_subtract("1.0", 1.0);

trace("//\"-1.0\" - 1.0");
assert_subtract("-1.0", 1.0);

trace("//\"0xFF1306\" - 1.0");
assert_subtract("0xFF1306", 1.0);

trace("//true - -1.0");
assert_subtract(true, -1.0);

trace("//false - -1.0");
assert_subtract(false, -1.0);

trace("//null - -1.0");
assert_subtract(null, -1.0);

trace("//undefined - -1.0");
assert_subtract(undefined, -1.0);

trace("//\"\" - -1.0");
assert_subtract("", -1.0);

trace("//\"str\" - -1.0");
assert_subtract("str", -1.0);

trace("//\"true\" - -1.0");
assert_subtract("true", -1.0);

trace("//\"false\" - -1.0");
assert_subtract("false", -1.0);

trace("//0.0 - -1.0");
assert_subtract(0.0, -1.0);

trace("//NaN - -1.0");
assert_subtract(NaN, -1.0);

trace("//-0.0 - -1.0");
assert_subtract(-0.0, -1.0);

trace("//Infinity - -1.0");
assert_subtract(Infinity, -1.0);

trace("//1.0 - -1.0");
assert_subtract(1.0, -1.0);

trace("//-1.0 - -1.0");
assert_subtract(-1.0, -1.0);

trace("//0xFF1306 - -1.0");
assert_subtract(0xFF1306, -1.0);

trace("//new Object() - -1.0");
assert_subtract({}, -1.0);

trace("//\"0.0\" - -1.0");
assert_subtract("0.0", -1.0);

trace("//\"NaN\" - -1.0");
assert_subtract("NaN", -1.0);

trace("//\"-0.0\" - -1.0");
assert_subtract("-0.0", -1.0);

trace("//\"Infinity\" - -1.0");
assert_subtract("Infinity", -1.0);

trace("//\"1.0\" - -1.0");
assert_subtract("1.0", -1.0);

trace("//\"-1.0\" - -1.0");
assert_subtract("-1.0", -1.0);

trace("//\"0xFF1306\" - -1.0");
assert_subtract("0xFF1306", -1.0);

trace("//true - 0xFF1306");
assert_subtract(true, 0xFF1306);

trace("//false - 0xFF1306");
assert_subtract(false, 0xFF1306);

trace("//null - 0xFF1306");
assert_subtract(null, 0xFF1306);

trace("//undefined - 0xFF1306");
assert_subtract(undefined, 0xFF1306);

trace("//\"\" - 0xFF1306");
assert_subtract("", 0xFF1306);

trace("//\"str\" - 0xFF1306");
assert_subtract("str", 0xFF1306);

trace("//\"true\" - 0xFF1306");
assert_subtract("true", 0xFF1306);

trace("//\"false\" - 0xFF1306");
assert_subtract("false", 0xFF1306);

trace("//0.0 - 0xFF1306");
assert_subtract(0.0, 0xFF1306);

trace("//NaN - 0xFF1306");
assert_subtract(NaN, 0xFF1306);

trace("//-0.0 - 0xFF1306");
assert_subtract(-0.0, 0xFF1306);

trace("//Infinity - 0xFF1306");
assert_subtract(Infinity, 0xFF1306);

trace("//1.0 - 0xFF1306");
assert_subtract(1.0, 0xFF1306);

trace("//-1.0 - 0xFF1306");
assert_subtract(-1.0, 0xFF1306);

trace("//0xFF1306 - 0xFF1306");
assert_subtract(0xFF1306, 0xFF1306);

trace("//new Object() - 0xFF1306");
assert_subtract({}, 0xFF1306);

trace("//\"0.0\" - 0xFF1306");
assert_subtract("0.0", 0xFF1306);

trace("//\"NaN\" - 0xFF1306");
assert_subtract("NaN", 0xFF1306);

trace("//\"-0.0\" - 0xFF1306");
assert_subtract("-0.0", 0xFF1306);

trace("//\"Infinity\" - 0xFF1306");
assert_subtract("Infinity", 0xFF1306);

trace("//\"1.0\" - 0xFF1306");
assert_subtract("1.0", 0xFF1306);

trace("//\"-1.0\" - 0xFF1306");
assert_subtract("-1.0", 0xFF1306);

trace("//\"0xFF1306\" - 0xFF1306");
assert_subtract("0xFF1306", 0xFF1306);

trace("//true - new Object()");
assert_subtract(true, {});

trace("//false - new Object()");
assert_subtract(false, {});

trace("//null - new Object()");
assert_subtract(null, {});

trace("//undefined - new Object()");
assert_subtract(undefined, {});

trace("//\"\" - new Object()");
assert_subtract("", {});

trace("//\"str\" - new Object()");
assert_subtract("str", {});

trace("//\"true\" - new Object()");
assert_subtract("true", {});

trace("//\"false\" - new Object()");
assert_subtract("false", {});

trace("//0.0 - new Object()");
assert_subtract(0.0, {});

trace("//NaN - new Object()");
assert_subtract(NaN, {});

trace("//-0.0 - new Object()");
assert_subtract(-0.0, {});

trace("//Infinity - new Object()");
assert_subtract(Infinity, {});

trace("//1.0 - new Object()");
assert_subtract(1.0, {});

trace("//-1.0 - new Object()");
assert_subtract(-1.0, {});

trace("//0xFF1306 - new Object()");
assert_subtract(0xFF1306, {});

trace("//new Object() - new Object()");
assert_subtract({}, {});

trace("//\"0.0\" - new Object()");
assert_subtract("0.0", {});

trace("//\"NaN\" - new Object()");
assert_subtract("NaN", {});

trace("//\"-0.0\" - new Object()");
assert_subtract("-0.0", {});

trace("//\"Infinity\" - new Object()");
assert_subtract("Infinity", {});

trace("//\"1.0\" - new Object()");
assert_subtract("1.0", {});

trace("//\"-1.0\" - new Object()");
assert_subtract("-1.0", {});

trace("//\"0xFF1306\" - new Object()");
assert_subtract("0xFF1306", {});

trace("//true - \"0.0\"");
assert_subtract(true, "0.0");

trace("//false - \"0.0\"");
assert_subtract(false, "0.0");

trace("//null - \"0.0\"");
assert_subtract(null, "0.0");

trace("//undefined - \"0.0\"");
assert_subtract(undefined, "0.0");

trace("//\"\" - \"0.0\"");
assert_subtract("", "0.0");

trace("//\"str\" - \"0.0\"");
assert_subtract("str", "0.0");

trace("//\"true\" - \"0.0\"");
assert_subtract("true", "0.0");

trace("//\"false\" - \"0.0\"");
assert_subtract("false", "0.0");

trace("//0.0 - \"0.0\"");
assert_subtract(0.0, "0.0");

trace("//NaN - \"0.0\"");
assert_subtract(NaN, "0.0");

trace("//-0.0 - \"0.0\"");
assert_subtract(-0.0, "0.0");

trace("//Infinity - \"0.0\"");
assert_subtract(Infinity, "0.0");

trace("//1.0 - \"0.0\"");
assert_subtract(1.0, "0.0");

trace("//-1.0 - \"0.0\"");
assert_subtract(-1.0, "0.0");

trace("//0xFF1306 - \"0.0\"");
assert_subtract(0xFF1306, "0.0");

trace("//new Object() - \"0.0\"");
assert_subtract({}, "0.0");

trace("//\"0.0\" - \"0.0\"");
assert_subtract("0.0", "0.0");

trace("//\"NaN\" - \"0.0\"");
assert_subtract("NaN", "0.0");

trace("//\"-0.0\" - \"0.0\"");
assert_subtract("-0.0", "0.0");

trace("//\"Infinity\" - \"0.0\"");
assert_subtract("Infinity", "0.0");

trace("//\"1.0\" - \"0.0\"");
assert_subtract("1.0", "0.0");

trace("//\"-1.0\" - \"0.0\"");
assert_subtract("-1.0", "0.0");

trace("//\"0xFF1306\" - \"0.0\"");
assert_subtract("0xFF1306", "0.0");

trace("//true - \"NaN\"");
assert_subtract(true, "NaN");

trace("//false - \"NaN\"");
assert_subtract(false, "NaN");

trace("//null - \"NaN\"");
assert_subtract(null, "NaN");

trace("//undefined - \"NaN\"");
assert_subtract(undefined, "NaN");

trace("//\"\" - \"NaN\"");
assert_subtract("", "NaN");

trace("//\"str\" - \"NaN\"");
assert_subtract("str", "NaN");

trace("//\"true\" - \"NaN\"");
assert_subtract("true", "NaN");

trace("//\"false\" - \"NaN\"");
assert_subtract("false", "NaN");

trace("//0.0 - \"NaN\"");
assert_subtract(0.0, "NaN");

trace("//NaN - \"NaN\"");
assert_subtract(NaN, "NaN");

trace("//-0.0 - \"NaN\"");
assert_subtract(-0.0, "NaN");

trace("//Infinity - \"NaN\"");
assert_subtract(Infinity, "NaN");

trace("//1.0 - \"NaN\"");
assert_subtract(1.0, "NaN");

trace("//-1.0 - \"NaN\"");
assert_subtract(-1.0, "NaN");

trace("//0xFF1306 - \"NaN\"");
assert_subtract(0xFF1306, "NaN");

trace("//new Object() - \"NaN\"");
assert_subtract({}, "NaN");

trace("//\"0.0\" - \"NaN\"");
assert_subtract("0.0", "NaN");

trace("//\"NaN\" - \"NaN\"");
assert_subtract("NaN", "NaN");

trace("//\"-0.0\" - \"NaN\"");
assert_subtract("-0.0", "NaN");

trace("//\"Infinity\" - \"NaN\"");
assert_subtract("Infinity", "NaN");

trace("//\"1.0\" - \"NaN\"");
assert_subtract("1.0", "NaN");

trace("//\"-1.0\" - \"NaN\"");
assert_subtract("-1.0", "NaN");

trace("//\"0xFF1306\" - \"NaN\"");
assert_subtract("0xFF1306", "NaN");

trace("//true - \"-0.0\"");
assert_subtract(true, "-0.0");

trace("//false - \"-0.0\"");
assert_subtract(false, "-0.0");

trace("//null - \"-0.0\"");
assert_subtract(null, "-0.0");

trace("//undefined - \"-0.0\"");
assert_subtract(undefined, "-0.0");

trace("//\"\" - \"-0.0\"");
assert_subtract("", "-0.0");

trace("//\"str\" - \"-0.0\"");
assert_subtract("str", "-0.0");

trace("//\"true\" - \"-0.0\"");
assert_subtract("true", "-0.0");

trace("//\"false\" - \"-0.0\"");
assert_subtract("false", "-0.0");

trace("//0.0 - \"-0.0\"");
assert_subtract(0.0, "-0.0");

trace("//NaN - \"-0.0\"");
assert_subtract(NaN, "-0.0");

trace("//-0.0 - \"-0.0\"");
assert_subtract(-0.0, "-0.0");

trace("//Infinity - \"-0.0\"");
assert_subtract(Infinity, "-0.0");

trace("//1.0 - \"-0.0\"");
assert_subtract(1.0, "-0.0");

trace("//-1.0 - \"-0.0\"");
assert_subtract(-1.0, "-0.0");

trace("//0xFF1306 - \"-0.0\"");
assert_subtract(0xFF1306, "-0.0");

trace("//new Object() - \"-0.0\"");
assert_subtract({}, "-0.0");

trace("//\"0.0\" - \"-0.0\"");
assert_subtract("0.0", "-0.0");

trace("//\"NaN\" - \"-0.0\"");
assert_subtract("NaN", "-0.0");

trace("//\"-0.0\" - \"-0.0\"");
assert_subtract("-0.0", "-0.0");

trace("//\"Infinity\" - \"-0.0\"");
assert_subtract("Infinity", "-0.0");

trace("//\"1.0\" - \"-0.0\"");
assert_subtract("1.0", "-0.0");

trace("//\"-1.0\" - \"-0.0\"");
assert_subtract("-1.0", "-0.0");

trace("//\"0xFF1306\" - \"-0.0\"");
assert_subtract("0xFF1306", "-0.0");

trace("//true - \"Infinity\"");
assert_subtract(true, "Infinity");

trace("//false - \"Infinity\"");
assert_subtract(false, "Infinity");

trace("//null - \"Infinity\"");
assert_subtract(null, "Infinity");

trace("//undefined - \"Infinity\"");
assert_subtract(undefined, "Infinity");

trace("//\"\" - \"Infinity\"");
assert_subtract("", "Infinity");

trace("//\"str\" - \"Infinity\"");
assert_subtract("str", "Infinity");

trace("//\"true\" - \"Infinity\"");
assert_subtract("true", "Infinity");

trace("//\"false\" - \"Infinity\"");
assert_subtract("false", "Infinity");

trace("//0.0 - \"Infinity\"");
assert_subtract(0.0, "Infinity");

trace("//NaN - \"Infinity\"");
assert_subtract(NaN, "Infinity");

trace("//-0.0 - \"Infinity\"");
assert_subtract(-0.0, "Infinity");

trace("//Infinity - \"Infinity\"");
assert_subtract(Infinity, "Infinity");

trace("//1.0 - \"Infinity\"");
assert_subtract(1.0, "Infinity");

trace("//-1.0 - \"Infinity\"");
assert_subtract(-1.0, "Infinity");

trace("//0xFF1306 - \"Infinity\"");
assert_subtract(0xFF1306, "Infinity");

trace("//new Object() - \"Infinity\"");
assert_subtract({}, "Infinity");

trace("//\"0.0\" - \"Infinity\"");
assert_subtract("0.0", "Infinity");

trace("//\"NaN\" - \"Infinity\"");
assert_subtract("NaN", "Infinity");

trace("//\"-0.0\" - \"Infinity\"");
assert_subtract("-0.0", "Infinity");

trace("//\"Infinity\" - \"Infinity\"");
assert_subtract("Infinity", "Infinity");

trace("//\"1.0\" - \"Infinity\"");
assert_subtract("1.0", "Infinity");

trace("//\"-1.0\" - \"Infinity\"");
assert_subtract("-1.0", "Infinity");

trace("//\"0xFF1306\" - \"Infinity\"");
assert_subtract("0xFF1306", "Infinity");

trace("//true - \"1.0\"");
assert_subtract(true, "1.0");

trace("//false - \"1.0\"");
assert_subtract(false, "1.0");

trace("//null - \"1.0\"");
assert_subtract(null, "1.0");

trace("//undefined - \"1.0\"");
assert_subtract(undefined, "1.0");

trace("//\"\" - \"1.0\"");
assert_subtract("", "1.0");

trace("//\"str\" - \"1.0\"");
assert_subtract("str", "1.0");

trace("//\"true\" - \"1.0\"");
assert_subtract("true", "1.0");

trace("//\"false\" - \"1.0\"");
assert_subtract("false", "1.0");

trace("//0.0 - \"1.0\"");
assert_subtract(0.0, "1.0");

trace("//NaN - \"1.0\"");
assert_subtract(NaN, "1.0");

trace("//-0.0 - \"1.0\"");
assert_subtract(-0.0, "1.0");

trace("//Infinity - \"1.0\"");
assert_subtract(Infinity, "1.0");

trace("//1.0 - \"1.0\"");
assert_subtract(1.0, "1.0");

trace("//-1.0 - \"1.0\"");
assert_subtract(-1.0, "1.0");

trace("//0xFF1306 - \"1.0\"");
assert_subtract(0xFF1306, "1.0");

trace("//new Object() - \"1.0\"");
assert_subtract({}, "1.0");

trace("//\"0.0\" - \"1.0\"");
assert_subtract("0.0", "1.0");

trace("//\"NaN\" - \"1.0\"");
assert_subtract("NaN", "1.0");

trace("//\"-0.0\" - \"1.0\"");
assert_subtract("-0.0", "1.0");

trace("//\"Infinity\" - \"1.0\"");
assert_subtract("Infinity", "1.0");

trace("//\"1.0\" - \"1.0\"");
assert_subtract("1.0", "1.0");

trace("//\"-1.0\" - \"1.0\"");
assert_subtract("-1.0", "1.0");

trace("//\"0xFF1306\" - \"1.0\"");
assert_subtract("0xFF1306", "1.0");

trace("//true - \"-1.0\"");
assert_subtract(true, "-1.0");

trace("//false - \"-1.0\"");
assert_subtract(false, "-1.0");

trace("//null - \"-1.0\"");
assert_subtract(null, "-1.0");

trace("//undefined - \"-1.0\"");
assert_subtract(undefined, "-1.0");

trace("//\"\" - \"-1.0\"");
assert_subtract("", "-1.0");

trace("//\"str\" - \"-1.0\"");
assert_subtract("str", "-1.0");

trace("//\"true\" - \"-1.0\"");
assert_subtract("true", "-1.0");

trace("//\"false\" - \"-1.0\"");
assert_subtract("false", "-1.0");

trace("//0.0 - \"-1.0\"");
assert_subtract(0.0, "-1.0");

trace("//NaN - \"-1.0\"");
assert_subtract(NaN, "-1.0");

trace("//-0.0 - \"-1.0\"");
assert_subtract(-0.0, "-1.0");

trace("//Infinity - \"-1.0\"");
assert_subtract(Infinity, "-1.0");

trace("//1.0 - \"-1.0\"");
assert_subtract(1.0, "-1.0");

trace("//-1.0 - \"-1.0\"");
assert_subtract(-1.0, "-1.0");

trace("//0xFF1306 - \"-1.0\"");
assert_subtract(0xFF1306, "-1.0");

trace("//new Object() - \"-1.0\"");
assert_subtract({}, "-1.0");

trace("//\"0.0\" - \"-1.0\"");
assert_subtract("0.0", "-1.0");

trace("//\"NaN\" - \"-1.0\"");
assert_subtract("NaN", "-1.0");

trace("//\"-0.0\" - \"-1.0\"");
assert_subtract("-0.0", "-1.0");

trace("//\"Infinity\" - \"-1.0\"");
assert_subtract("Infinity", "-1.0");

trace("//\"1.0\" - \"-1.0\"");
assert_subtract("1.0", "-1.0");

trace("//\"-1.0\" - \"-1.0\"");
assert_subtract("-1.0", "-1.0");

trace("//\"0xFF1306\" - \"-1.0\"");
assert_subtract("0xFF1306", "-1.0");

trace("//true - \"0xFF1306\"");
assert_subtract(true, "0xFF1306");

trace("//false - \"0xFF1306\"");
assert_subtract(false, "0xFF1306");

trace("//null - \"0xFF1306\"");
assert_subtract(null, "0xFF1306");

trace("//undefined - \"0xFF1306\"");
assert_subtract(undefined, "0xFF1306");

trace("//\"\" - \"0xFF1306\"");
assert_subtract("", "0xFF1306");

trace("//\"str\" - \"0xFF1306\"");
assert_subtract("str", "0xFF1306");

trace("//\"true\" - \"0xFF1306\"");
assert_subtract("true", "0xFF1306");

trace("//\"false\" - \"0xFF1306\"");
assert_subtract("false", "0xFF1306");

trace("//0.0 - \"0xFF1306\"");
assert_subtract(0.0, "0xFF1306");

trace("//NaN - \"0xFF1306\"");
assert_subtract(NaN, "0xFF1306");

trace("//-0.0 - \"0xFF1306\"");
assert_subtract(-0.0, "0xFF1306");

trace("//Infinity - \"0xFF1306\"");
assert_subtract(Infinity, "0xFF1306");

trace("//1.0 - \"0xFF1306\"");
assert_subtract(1.0, "0xFF1306");

trace("//-1.0 - \"0xFF1306\"");
assert_subtract(-1.0, "0xFF1306");

trace("//0xFF1306 - \"0xFF1306\"");
assert_subtract(0xFF1306, "0xFF1306");

trace("//new Object() - \"0xFF1306\"");
assert_subtract({}, "0xFF1306");

trace("//\"0.0\" - \"0xFF1306\"");
assert_subtract("0.0", "0xFF1306");

trace("//\"NaN\" - \"0xFF1306\"");
assert_subtract("NaN", "0xFF1306");

trace("//\"-0.0\" - \"0xFF1306\"");
assert_subtract("-0.0", "0xFF1306");

trace("//\"Infinity\" - \"0xFF1306\"");
assert_subtract("Infinity", "0xFF1306");

trace("//\"1.0\" - \"0xFF1306\"");
assert_subtract("1.0", "0xFF1306");

trace("//\"-1.0\" - \"0xFF1306\"");
assert_subtract("-1.0", "0xFF1306");

trace("//\"0xFF1306\" - \"0xFF1306\"");
assert_subtract("0xFF1306", "0xFF1306");