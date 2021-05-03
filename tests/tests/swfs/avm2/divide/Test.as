package {
	public class Test {
	}
}

function assert_divide(val1, val2) {
	trace(val1 / val2);
}

trace("//true / true");
assert_divide(true, true);

trace("//false / true");
assert_divide(false, true);

trace("//null / true");
assert_divide(null, true);

trace("//undefined / true");
assert_divide(undefined, true);

trace("//\"\" / true");
assert_divide("", true);

trace("//\"str\" / true");
assert_divide("str", true);

trace("//\"true\" / true");
assert_divide("true", true);

trace("//\"false\" / true");
assert_divide("false", true);

trace("//0.0 / true");
assert_divide(0.0, true);

trace("//NaN / true");
assert_divide(NaN, true);

trace("//-0.0 / true");
assert_divide(-0.0, true);

trace("//Infinity / true");
assert_divide(Infinity, true);

trace("//1.0 / true");
assert_divide(1.0, true);

trace("//-1.0 / true");
assert_divide(-1.0, true);

trace("//0xFF1306 / true");
assert_divide(0xFF1306, true);

trace("//new Object() / true");
assert_divide({}, true);

trace("//\"0.0\" / true");
assert_divide("0.0", true);

trace("//\"NaN\" / true");
assert_divide("NaN", true);

trace("//\"-0.0\" / true");
assert_divide("-0.0", true);

trace("//\"Infinity\" / true");
assert_divide("Infinity", true);

trace("//\"1.0\" / true");
assert_divide("1.0", true);

trace("//\"-1.0\" / true");
assert_divide("-1.0", true);

trace("//\"0xFF1306\" / true");
assert_divide("0xFF1306", true);

trace("//true / false");
assert_divide(true, false);

trace("//false / false");
assert_divide(false, false);

trace("//null / false");
assert_divide(null, false);

trace("//undefined / false");
assert_divide(undefined, false);

trace("//\"\" / false");
assert_divide("", false);

trace("//\"str\" / false");
assert_divide("str", false);

trace("//\"true\" / false");
assert_divide("true", false);

trace("//\"false\" / false");
assert_divide("false", false);

trace("//0.0 / false");
assert_divide(0.0, false);

trace("//NaN / false");
assert_divide(NaN, false);

trace("//-0.0 / false");
assert_divide(-0.0, false);

trace("//Infinity / false");
assert_divide(Infinity, false);

trace("//1.0 / false");
assert_divide(1.0, false);

trace("//-1.0 / false");
assert_divide(-1.0, false);

trace("//0xFF1306 / false");
assert_divide(0xFF1306, false);

trace("//new Object() / false");
assert_divide({}, false);

trace("//\"0.0\" / false");
assert_divide("0.0", false);

trace("//\"NaN\" / false");
assert_divide("NaN", false);

trace("//\"-0.0\" / false");
assert_divide("-0.0", false);

trace("//\"Infinity\" / false");
assert_divide("Infinity", false);

trace("//\"1.0\" / false");
assert_divide("1.0", false);

trace("//\"-1.0\" / false");
assert_divide("-1.0", false);

trace("//\"0xFF1306\" / false");
assert_divide("0xFF1306", false);
trace("//true / null");
assert_divide(true, null);

trace("//false / null");
assert_divide(false, null);

trace("//null / null");
assert_divide(null, null);

trace("//undefined / null");
assert_divide(undefined, null);

trace("//\"\" / null");
assert_divide("", null);

trace("//\"str\" / null");
assert_divide("str", null);

trace("//\"true\" / null");
assert_divide("true", null);

trace("//\"false\" / null");
assert_divide("false", null);

trace("//0.0 / null");
assert_divide(0.0, null);

trace("//NaN / null");
assert_divide(NaN, null);

trace("//-0.0 / null");
assert_divide(-0.0, null);

trace("//Infinity / null");
assert_divide(Infinity, null);

trace("//1.0 / null");
assert_divide(1.0, null);

trace("//-1.0 / null");
assert_divide(-1.0, null);

trace("//0xFF1306 / null");
assert_divide(0xFF1306, null);

trace("//new Object() / null");
assert_divide({}, null);

trace("//\"0.0\" / null");
assert_divide("0.0", null);

trace("//\"NaN\" / null");
assert_divide("NaN", null);

trace("//\"-0.0\" / null");
assert_divide("-0.0", null);

trace("//\"Infinity\" / null");
assert_divide("Infinity", null);

trace("//\"1.0\" / null");
assert_divide("1.0", null);

trace("//\"-1.0\" / null");
assert_divide("-1.0", null);

trace("//\"0xFF1306\" / null");
assert_divide("0xFF1306", null);

trace("//true / undefined");
assert_divide(true, undefined);

trace("//false / undefined");
assert_divide(false, undefined);

trace("//null / undefined");
assert_divide(null, undefined);

trace("//undefined / undefined");
assert_divide(undefined, undefined);

trace("//\"\" / undefined");
assert_divide("", undefined);

trace("//\"str\" / undefined");
assert_divide("str", undefined);

trace("//\"true\" / undefined");
assert_divide("true", undefined);

trace("//\"false\" / undefined");
assert_divide("false", undefined);

trace("//0.0 / undefined");
assert_divide(0.0, undefined);

trace("//NaN / undefined");
assert_divide(NaN, undefined);

trace("//-0.0 / undefined");
assert_divide(-0.0, undefined);

trace("//Infinity / undefined");
assert_divide(Infinity, undefined);

trace("//1.0 / undefined");
assert_divide(1.0, undefined);

trace("//-1.0 / undefined");
assert_divide(-1.0, undefined);

trace("//0xFF1306 / undefined");
assert_divide(0xFF1306, undefined);

trace("//new Object() / undefined");
assert_divide({}, undefined);

trace("//\"0.0\" / undefined");
assert_divide("0.0", undefined);

trace("//\"NaN\" / undefined");
assert_divide("NaN", undefined);

trace("//\"-0.0\" / undefined");
assert_divide("-0.0", undefined);

trace("//\"Infinity\" / undefined");
assert_divide("Infinity", undefined);

trace("//\"1.0\" / undefined");
assert_divide("1.0", undefined);

trace("//\"-1.0\" / undefined");
assert_divide("-1.0", undefined);

trace("//\"0xFF1306\" / undefined");
assert_divide("0xFF1306", undefined);

trace("//true / \"\"");
assert_divide(true, "");

trace("//false / \"\"");
assert_divide(false, "");

trace("//null / \"\"");
assert_divide(null, "");

trace("//undefined / \"\"");
assert_divide(undefined, "");

trace("//\"\" / \"\"");
assert_divide("", "");

trace("//\"str\" / \"\"");
assert_divide("str", "");

trace("//\"true\" / \"\"");
assert_divide("true", "");

trace("//\"false\" / \"\"");
assert_divide("false", "");

trace("//0.0 / \"\"");
assert_divide(0.0, "");

trace("//NaN / \"\"");
assert_divide(NaN, "");

trace("//-0.0 / \"\"");
assert_divide(-0.0, "");

trace("//Infinity / \"\"");
assert_divide(Infinity, "");

trace("//1.0 / \"\"");
assert_divide(1.0, "");

trace("//-1.0 / \"\"");
assert_divide(-1.0, "");

trace("//0xFF1306 / \"\"");
assert_divide(0xFF1306, "");

trace("//new Object() / \"\"");
assert_divide({}, "");

trace("//\"0.0\" / \"\"");
assert_divide("0.0", "");

trace("//\"NaN\" / \"\"");
assert_divide("NaN", "");

trace("//\"-0.0\" / \"\"");
assert_divide("-0.0", "");

trace("//\"Infinity\" / \"\"");
assert_divide("Infinity", "");

trace("//\"1.0\" / \"\"");
assert_divide("1.0", "");

trace("//\"-1.0\" / \"\"");
assert_divide("-1.0", "");

trace("//\"0xFF1306\" / \"\"");
assert_divide("0xFF1306", "");

trace("//true / \"str\"");
assert_divide(true, "str");

trace("//false / \"str\"");
assert_divide(false, "str");

trace("//null / \"str\"");
assert_divide(null, "str");

trace("//undefined / \"str\"");
assert_divide(undefined, "str");

trace("//\"\" / \"str\"");
assert_divide("", "str");

trace("//\"str\" / \"str\"");
assert_divide("str", "str");

trace("//\"true\" / \"str\"");
assert_divide("true", "str");

trace("//\"false\" / \"str\"");
assert_divide("false", "str");

trace("//0.0 / \"str\"");
assert_divide(0.0, "str");

trace("//NaN / \"str\"");
assert_divide(NaN, "str");

trace("//-0.0 / \"str\"");
assert_divide(-0.0, "str");

trace("//Infinity / \"str\"");
assert_divide(Infinity, "str");

trace("//1.0 / \"str\"");
assert_divide(1.0, "str");

trace("//-1.0 / \"str\"");
assert_divide(-1.0, "str");

trace("//0xFF1306 / \"str\"");
assert_divide(0xFF1306, "str");

trace("//new Object() / \"str\"");
assert_divide({}, "str");

trace("//\"0.0\" / \"str\"");
assert_divide("0.0", "str");

trace("//\"NaN\" / \"str\"");
assert_divide("NaN", "str");

trace("//\"-0.0\" / \"str\"");
assert_divide("-0.0", "str");

trace("//\"Infinity\" / \"str\"");
assert_divide("Infinity", "str");

trace("//\"1.0\" / \"str\"");
assert_divide("1.0", "str");

trace("//\"-1.0\" / \"str\"");
assert_divide("-1.0", "str");

trace("//\"0xFF1306\" / \"str\"");
assert_divide("0xFF1306", "str");

trace("//true / \"true\"");
assert_divide(true, "true");

trace("//false / \"true\"");
assert_divide(false, "true");

trace("//null / \"true\"");
assert_divide(null, "true");

trace("//undefined / \"true\"");
assert_divide(undefined, "true");

trace("//\"\" / \"true\"");
assert_divide("", "true");

trace("//\"str\" / \"true\"");
assert_divide("str", "true");

trace("//\"true\" / \"true\"");
assert_divide("true", "true");

trace("//\"false\" / \"true\"");
assert_divide("false", "true");

trace("//0.0 / \"true\"");
assert_divide(0.0, "true");

trace("//NaN / \"true\"");
assert_divide(NaN, "true");

trace("//-0.0 / \"true\"");
assert_divide(-0.0, "true");

trace("//Infinity / \"true\"");
assert_divide(Infinity, "true");

trace("//1.0 / \"true\"");
assert_divide(1.0, "true");

trace("//-1.0 / \"true\"");
assert_divide(-1.0, "true");

trace("//0xFF1306 / \"true\"");
assert_divide(0xFF1306, "true");

trace("//new Object() / \"true\"");
assert_divide({}, "true");

trace("//\"0.0\" / \"true\"");
assert_divide("0.0", "true");

trace("//\"NaN\" / \"true\"");
assert_divide("NaN", "true");

trace("//\"-0.0\" / \"true\"");
assert_divide("-0.0", "true");

trace("//\"Infinity\" / \"true\"");
assert_divide("Infinity", "true");

trace("//\"1.0\" / \"true\"");
assert_divide("1.0", "true");

trace("//\"-1.0\" / \"true\"");
assert_divide("-1.0", "true");

trace("//\"0xFF1306\" / \"true\"");
assert_divide("0xFF1306", "true");

trace("//true / \"false\"");
assert_divide(true, "false");

trace("//false / \"false\"");
assert_divide(false, "false");

trace("//null / \"false\"");
assert_divide(null, "false");

trace("//undefined / \"false\"");
assert_divide(undefined, "false");

trace("//\"\" / \"false\"");
assert_divide("", "false");

trace("//\"str\" / \"false\"");
assert_divide("str", "false");

trace("//\"true\" / \"false\"");
assert_divide("true", "false");

trace("//\"false\" / \"false\"");
assert_divide("false", "false");

trace("//0.0 / \"false\"");
assert_divide(0.0, "false");

trace("//NaN / \"false\"");
assert_divide(NaN, "false");

trace("//-0.0 / \"false\"");
assert_divide(-0.0, "false");

trace("//Infinity / \"false\"");
assert_divide(Infinity, "false");

trace("//1.0 / \"false\"");
assert_divide(1.0, "false");

trace("//-1.0 / \"false\"");
assert_divide(-1.0, "false");

trace("//0xFF1306 / \"false\"");
assert_divide(0xFF1306, "false");

trace("//new Object() / \"false\"");
assert_divide({}, "false");

trace("//\"0.0\" / \"false\"");
assert_divide("0.0", "false");

trace("//\"NaN\" / \"false\"");
assert_divide("NaN", "false");

trace("//\"-0.0\" / \"false\"");
assert_divide("-0.0", "false");

trace("//\"Infinity\" / \"false\"");
assert_divide("Infinity", "false");

trace("//\"1.0\" / \"false\"");
assert_divide("1.0", "false");

trace("//\"-1.0\" / \"false\"");
assert_divide("-1.0", "false");

trace("//\"0xFF1306\" / \"false\"");
assert_divide("0xFF1306", "false");

trace("//true / 0.0");
assert_divide(true, 0.0);

trace("//false / 0.0");
assert_divide(false, 0.0);

trace("//null / 0.0");
assert_divide(null, 0.0);

trace("//undefined / 0.0");
assert_divide(undefined, 0.0);

trace("//\"\" / 0.0");
assert_divide("", 0.0);

trace("//\"str\" / 0.0");
assert_divide("str", 0.0);

trace("//\"true\" / 0.0");
assert_divide("true", 0.0);

trace("//\"false\" / 0.0");
assert_divide("false", 0.0);

trace("//0.0 / 0.0");
assert_divide(0.0, 0.0);

trace("//NaN / 0.0");
assert_divide(NaN, 0.0);

trace("//-0.0 / 0.0");
assert_divide(-0.0, 0.0);

trace("//Infinity / 0.0");
assert_divide(Infinity, 0.0);

trace("//1.0 / 0.0");
assert_divide(1.0, 0.0);

trace("//-1.0 / 0.0");
assert_divide(-1.0, 0.0);

trace("//0xFF1306 / 0.0");
assert_divide(0xFF1306, 0.0);

trace("//new Object() / 0.0");
assert_divide({}, 0.0);

trace("//\"0.0\" / 0.0");
assert_divide("0.0", 0.0);

trace("//\"NaN\" / 0.0");
assert_divide("NaN", 0.0);

trace("//\"-0.0\" / 0.0");
assert_divide("-0.0", 0.0);

trace("//\"Infinity\" / 0.0");
assert_divide("Infinity", 0.0);

trace("//\"1.0\" / 0.0");
assert_divide("1.0", 0.0);

trace("//\"-1.0\" / 0.0");
assert_divide("-1.0", 0.0);

trace("//\"0xFF1306\" / 0.0");
assert_divide("0xFF1306", 0.0);

trace("//true / NaN");
assert_divide(true, NaN);

trace("//false / NaN");
assert_divide(false, NaN);

trace("//null / NaN");
assert_divide(null, NaN);

trace("//undefined / NaN");
assert_divide(undefined, NaN);

trace("//\"\" / NaN");
assert_divide("", NaN);

trace("//\"str\" / NaN");
assert_divide("str", NaN);

trace("//\"true\" / NaN");
assert_divide("true", NaN);

trace("//\"false\" / NaN");
assert_divide("false", NaN);

trace("//0.0 / NaN");
assert_divide(0.0, NaN);

trace("//NaN / NaN");
assert_divide(NaN, NaN);

trace("//-0.0 / NaN");
assert_divide(-0.0, NaN);

trace("//Infinity / NaN");
assert_divide(Infinity, NaN);

trace("//1.0 / NaN");
assert_divide(1.0, NaN);

trace("//-1.0 / NaN");
assert_divide(-1.0, NaN);

trace("//0xFF1306 / NaN");
assert_divide(0xFF1306, NaN);

trace("//new Object() / NaN");
assert_divide({}, NaN);

trace("//\"0.0\" / NaN");
assert_divide("0.0", NaN);

trace("//\"NaN\" / NaN");
assert_divide("NaN", NaN);

trace("//\"-0.0\" / NaN");
assert_divide("-0.0", NaN);

trace("//\"Infinity\" / NaN");
assert_divide("Infinity", NaN);

trace("//\"1.0\" / NaN");
assert_divide("1.0", NaN);

trace("//\"-1.0\" / NaN");
assert_divide("-1.0", NaN);

trace("//\"0xFF1306\" / NaN");
assert_divide("0xFF1306", NaN);

trace("//true / -0.0");
assert_divide(true, -0.0);

trace("//false / -0.0");
assert_divide(false, -0.0);

trace("//null / -0.0");
assert_divide(null, -0.0);

trace("//undefined / -0.0");
assert_divide(undefined, -0.0);

trace("//\"\" / -0.0");
assert_divide("", -0.0);

trace("//\"str\" / -0.0");
assert_divide("str", -0.0);

trace("//\"true\" / -0.0");
assert_divide("true", -0.0);

trace("//\"false\" / -0.0");
assert_divide("false", -0.0);

trace("//0.0 / -0.0");
assert_divide(0.0, -0.0);

trace("//NaN / -0.0");
assert_divide(NaN, -0.0);

trace("//-0.0 / -0.0");
assert_divide(-0.0, -0.0);

trace("//Infinity / -0.0");
assert_divide(Infinity, -0.0);

trace("//1.0 / -0.0");
assert_divide(1.0, -0.0);

trace("//-1.0 / -0.0");
assert_divide(-1.0, -0.0);

trace("//0xFF1306 / -0.0");
assert_divide(0xFF1306, -0.0);

trace("//new Object() / -0.0");
assert_divide({}, -0.0);

trace("//\"0.0\" / -0.0");
assert_divide("0.0", -0.0);

trace("//\"NaN\" / -0.0");
assert_divide("NaN", -0.0);

trace("//\"-0.0\" / -0.0");
assert_divide("-0.0", -0.0);

trace("//\"Infinity\" / -0.0");
assert_divide("Infinity", -0.0);

trace("//\"1.0\" / -0.0");
assert_divide("1.0", -0.0);

trace("//\"-1.0\" / -0.0");
assert_divide("-1.0", -0.0);

trace("//\"0xFF1306\" / -0.0");
assert_divide("0xFF1306", -0.0);

trace("//true / Infinity");
assert_divide(true, Infinity);

trace("//false / Infinity");
assert_divide(false, Infinity);

trace("//null / Infinity");
assert_divide(null, Infinity);

trace("//undefined / Infinity");
assert_divide(undefined, Infinity);

trace("//\"\" / Infinity");
assert_divide("", Infinity);

trace("//\"str\" / Infinity");
assert_divide("str", Infinity);

trace("//\"true\" / Infinity");
assert_divide("true", Infinity);

trace("//\"false\" / Infinity");
assert_divide("false", Infinity);

trace("//0.0 / Infinity");
assert_divide(0.0, Infinity);

trace("//NaN / Infinity");
assert_divide(NaN, Infinity);

trace("//-0.0 / Infinity");
assert_divide(-0.0, Infinity);

trace("//Infinity / Infinity");
assert_divide(Infinity, Infinity);

trace("//1.0 / Infinity");
assert_divide(1.0, Infinity);

trace("//-1.0 / Infinity");
assert_divide(-1.0, Infinity);

trace("//0xFF1306 / Infinity");
assert_divide(0xFF1306, Infinity);

trace("//new Object() / Infinity");
assert_divide({}, Infinity);

trace("//\"0.0\" / Infinity");
assert_divide("0.0", Infinity);

trace("//\"NaN\" / Infinity");
assert_divide("NaN", Infinity);

trace("//\"-0.0\" / Infinity");
assert_divide("-0.0", Infinity);

trace("//\"Infinity\" / Infinity");
assert_divide("Infinity", Infinity);

trace("//\"1.0\" / Infinity");
assert_divide("1.0", Infinity);

trace("//\"-1.0\" / Infinity");
assert_divide("-1.0", Infinity);

trace("//\"0xFF1306\" / Infinity");
assert_divide("0xFF1306", Infinity);

trace("//true / 1.0");
assert_divide(true, 1.0);

trace("//false / 1.0");
assert_divide(false, 1.0);

trace("//null / 1.0");
assert_divide(null, 1.0);

trace("//undefined / 1.0");
assert_divide(undefined, 1.0);

trace("//\"\" / 1.0");
assert_divide("", 1.0);

trace("//\"str\" / 1.0");
assert_divide("str", 1.0);

trace("//\"true\" / 1.0");
assert_divide("true", 1.0);

trace("//\"false\" / 1.0");
assert_divide("false", 1.0);

trace("//0.0 / 1.0");
assert_divide(0.0, 1.0);

trace("//NaN / 1.0");
assert_divide(NaN, 1.0);

trace("//-0.0 / 1.0");
assert_divide(-0.0, 1.0);

trace("//Infinity / 1.0");
assert_divide(Infinity, 1.0);

trace("//1.0 / 1.0");
assert_divide(1.0, 1.0);

trace("//-1.0 / 1.0");
assert_divide(-1.0, 1.0);

trace("//0xFF1306 / 1.0");
assert_divide(0xFF1306, 1.0);

trace("//new Object() / 1.0");
assert_divide({}, 1.0);

trace("//\"0.0\" / 1.0");
assert_divide("0.0", 1.0);

trace("//\"NaN\" / 1.0");
assert_divide("NaN", 1.0);

trace("//\"-0.0\" / 1.0");
assert_divide("-0.0", 1.0);

trace("//\"Infinity\" / 1.0");
assert_divide("Infinity", 1.0);

trace("//\"1.0\" / 1.0");
assert_divide("1.0", 1.0);

trace("//\"-1.0\" / 1.0");
assert_divide("-1.0", 1.0);

trace("//\"0xFF1306\" / 1.0");
assert_divide("0xFF1306", 1.0);

trace("//true / -1.0");
assert_divide(true, -1.0);

trace("//false / -1.0");
assert_divide(false, -1.0);

trace("//null / -1.0");
assert_divide(null, -1.0);

trace("//undefined / -1.0");
assert_divide(undefined, -1.0);

trace("//\"\" / -1.0");
assert_divide("", -1.0);

trace("//\"str\" / -1.0");
assert_divide("str", -1.0);

trace("//\"true\" / -1.0");
assert_divide("true", -1.0);

trace("//\"false\" / -1.0");
assert_divide("false", -1.0);

trace("//0.0 / -1.0");
assert_divide(0.0, -1.0);

trace("//NaN / -1.0");
assert_divide(NaN, -1.0);

trace("//-0.0 / -1.0");
assert_divide(-0.0, -1.0);

trace("//Infinity / -1.0");
assert_divide(Infinity, -1.0);

trace("//1.0 / -1.0");
assert_divide(1.0, -1.0);

trace("//-1.0 / -1.0");
assert_divide(-1.0, -1.0);

trace("//0xFF1306 / -1.0");
assert_divide(0xFF1306, -1.0);

trace("//new Object() / -1.0");
assert_divide({}, -1.0);

trace("//\"0.0\" / -1.0");
assert_divide("0.0", -1.0);

trace("//\"NaN\" / -1.0");
assert_divide("NaN", -1.0);

trace("//\"-0.0\" / -1.0");
assert_divide("-0.0", -1.0);

trace("//\"Infinity\" / -1.0");
assert_divide("Infinity", -1.0);

trace("//\"1.0\" / -1.0");
assert_divide("1.0", -1.0);

trace("//\"-1.0\" / -1.0");
assert_divide("-1.0", -1.0);

trace("//\"0xFF1306\" / -1.0");
assert_divide("0xFF1306", -1.0);

trace("//true / 0xFF1306");
assert_divide(true, 0xFF1306);

trace("//false / 0xFF1306");
assert_divide(false, 0xFF1306);

trace("//null / 0xFF1306");
assert_divide(null, 0xFF1306);

trace("//undefined / 0xFF1306");
assert_divide(undefined, 0xFF1306);

trace("//\"\" / 0xFF1306");
assert_divide("", 0xFF1306);

trace("//\"str\" / 0xFF1306");
assert_divide("str", 0xFF1306);

trace("//\"true\" / 0xFF1306");
assert_divide("true", 0xFF1306);

trace("//\"false\" / 0xFF1306");
assert_divide("false", 0xFF1306);

trace("//0.0 / 0xFF1306");
assert_divide(0.0, 0xFF1306);

trace("//NaN / 0xFF1306");
assert_divide(NaN, 0xFF1306);

trace("//-0.0 / 0xFF1306");
assert_divide(-0.0, 0xFF1306);

trace("//Infinity / 0xFF1306");
assert_divide(Infinity, 0xFF1306);

trace("//1.0 / 0xFF1306");
assert_divide(1.0, 0xFF1306);

trace("//-1.0 / 0xFF1306");
assert_divide(-1.0, 0xFF1306);

trace("//0xFF1306 / 0xFF1306");
assert_divide(0xFF1306, 0xFF1306);

trace("//new Object() / 0xFF1306");
assert_divide({}, 0xFF1306);

trace("//\"0.0\" / 0xFF1306");
assert_divide("0.0", 0xFF1306);

trace("//\"NaN\" / 0xFF1306");
assert_divide("NaN", 0xFF1306);

trace("//\"-0.0\" / 0xFF1306");
assert_divide("-0.0", 0xFF1306);

trace("//\"Infinity\" / 0xFF1306");
assert_divide("Infinity", 0xFF1306);

trace("//\"1.0\" / 0xFF1306");
assert_divide("1.0", 0xFF1306);

trace("//\"-1.0\" / 0xFF1306");
assert_divide("-1.0", 0xFF1306);

trace("//\"0xFF1306\" / 0xFF1306");
assert_divide("0xFF1306", 0xFF1306);

trace("//true / new Object()");
assert_divide(true, {});

trace("//false / new Object()");
assert_divide(false, {});

trace("//null / new Object()");
assert_divide(null, {});

trace("//undefined / new Object()");
assert_divide(undefined, {});

trace("//\"\" / new Object()");
assert_divide("", {});

trace("//\"str\" / new Object()");
assert_divide("str", {});

trace("//\"true\" / new Object()");
assert_divide("true", {});

trace("//\"false\" / new Object()");
assert_divide("false", {});

trace("//0.0 / new Object()");
assert_divide(0.0, {});

trace("//NaN / new Object()");
assert_divide(NaN, {});

trace("//-0.0 / new Object()");
assert_divide(-0.0, {});

trace("//Infinity / new Object()");
assert_divide(Infinity, {});

trace("//1.0 / new Object()");
assert_divide(1.0, {});

trace("//-1.0 / new Object()");
assert_divide(-1.0, {});

trace("//0xFF1306 / new Object()");
assert_divide(0xFF1306, {});

trace("//new Object() / new Object()");
assert_divide({}, {});

trace("//\"0.0\" / new Object()");
assert_divide("0.0", {});

trace("//\"NaN\" / new Object()");
assert_divide("NaN", {});

trace("//\"-0.0\" / new Object()");
assert_divide("-0.0", {});

trace("//\"Infinity\" / new Object()");
assert_divide("Infinity", {});

trace("//\"1.0\" / new Object()");
assert_divide("1.0", {});

trace("//\"-1.0\" / new Object()");
assert_divide("-1.0", {});

trace("//\"0xFF1306\" / new Object()");
assert_divide("0xFF1306", {});

trace("//true / \"0.0\"");
assert_divide(true, "0.0");

trace("//false / \"0.0\"");
assert_divide(false, "0.0");

trace("//null / \"0.0\"");
assert_divide(null, "0.0");

trace("//undefined / \"0.0\"");
assert_divide(undefined, "0.0");

trace("//\"\" / \"0.0\"");
assert_divide("", "0.0");

trace("//\"str\" / \"0.0\"");
assert_divide("str", "0.0");

trace("//\"true\" / \"0.0\"");
assert_divide("true", "0.0");

trace("//\"false\" / \"0.0\"");
assert_divide("false", "0.0");

trace("//0.0 / \"0.0\"");
assert_divide(0.0, "0.0");

trace("//NaN / \"0.0\"");
assert_divide(NaN, "0.0");

trace("//-0.0 / \"0.0\"");
assert_divide(-0.0, "0.0");

trace("//Infinity / \"0.0\"");
assert_divide(Infinity, "0.0");

trace("//1.0 / \"0.0\"");
assert_divide(1.0, "0.0");

trace("//-1.0 / \"0.0\"");
assert_divide(-1.0, "0.0");

trace("//0xFF1306 / \"0.0\"");
assert_divide(0xFF1306, "0.0");

trace("//new Object() / \"0.0\"");
assert_divide({}, "0.0");

trace("//\"0.0\" / \"0.0\"");
assert_divide("0.0", "0.0");

trace("//\"NaN\" / \"0.0\"");
assert_divide("NaN", "0.0");

trace("//\"-0.0\" / \"0.0\"");
assert_divide("-0.0", "0.0");

trace("//\"Infinity\" / \"0.0\"");
assert_divide("Infinity", "0.0");

trace("//\"1.0\" / \"0.0\"");
assert_divide("1.0", "0.0");

trace("//\"-1.0\" / \"0.0\"");
assert_divide("-1.0", "0.0");

trace("//\"0xFF1306\" / \"0.0\"");
assert_divide("0xFF1306", "0.0");

trace("//true / \"NaN\"");
assert_divide(true, "NaN");

trace("//false / \"NaN\"");
assert_divide(false, "NaN");

trace("//null / \"NaN\"");
assert_divide(null, "NaN");

trace("//undefined / \"NaN\"");
assert_divide(undefined, "NaN");

trace("//\"\" / \"NaN\"");
assert_divide("", "NaN");

trace("//\"str\" / \"NaN\"");
assert_divide("str", "NaN");

trace("//\"true\" / \"NaN\"");
assert_divide("true", "NaN");

trace("//\"false\" / \"NaN\"");
assert_divide("false", "NaN");

trace("//0.0 / \"NaN\"");
assert_divide(0.0, "NaN");

trace("//NaN / \"NaN\"");
assert_divide(NaN, "NaN");

trace("//-0.0 / \"NaN\"");
assert_divide(-0.0, "NaN");

trace("//Infinity / \"NaN\"");
assert_divide(Infinity, "NaN");

trace("//1.0 / \"NaN\"");
assert_divide(1.0, "NaN");

trace("//-1.0 / \"NaN\"");
assert_divide(-1.0, "NaN");

trace("//0xFF1306 / \"NaN\"");
assert_divide(0xFF1306, "NaN");

trace("//new Object() / \"NaN\"");
assert_divide({}, "NaN");

trace("//\"0.0\" / \"NaN\"");
assert_divide("0.0", "NaN");

trace("//\"NaN\" / \"NaN\"");
assert_divide("NaN", "NaN");

trace("//\"-0.0\" / \"NaN\"");
assert_divide("-0.0", "NaN");

trace("//\"Infinity\" / \"NaN\"");
assert_divide("Infinity", "NaN");

trace("//\"1.0\" / \"NaN\"");
assert_divide("1.0", "NaN");

trace("//\"-1.0\" / \"NaN\"");
assert_divide("-1.0", "NaN");

trace("//\"0xFF1306\" / \"NaN\"");
assert_divide("0xFF1306", "NaN");

trace("//true / \"-0.0\"");
assert_divide(true, "-0.0");

trace("//false / \"-0.0\"");
assert_divide(false, "-0.0");

trace("//null / \"-0.0\"");
assert_divide(null, "-0.0");

trace("//undefined / \"-0.0\"");
assert_divide(undefined, "-0.0");

trace("//\"\" / \"-0.0\"");
assert_divide("", "-0.0");

trace("//\"str\" / \"-0.0\"");
assert_divide("str", "-0.0");

trace("//\"true\" / \"-0.0\"");
assert_divide("true", "-0.0");

trace("//\"false\" / \"-0.0\"");
assert_divide("false", "-0.0");

trace("//0.0 / \"-0.0\"");
assert_divide(0.0, "-0.0");

trace("//NaN / \"-0.0\"");
assert_divide(NaN, "-0.0");

trace("//-0.0 / \"-0.0\"");
assert_divide(-0.0, "-0.0");

trace("//Infinity / \"-0.0\"");
assert_divide(Infinity, "-0.0");

trace("//1.0 / \"-0.0\"");
assert_divide(1.0, "-0.0");

trace("//-1.0 / \"-0.0\"");
assert_divide(-1.0, "-0.0");

trace("//0xFF1306 / \"-0.0\"");
assert_divide(0xFF1306, "-0.0");

trace("//new Object() / \"-0.0\"");
assert_divide({}, "-0.0");

trace("//\"0.0\" / \"-0.0\"");
assert_divide("0.0", "-0.0");

trace("//\"NaN\" / \"-0.0\"");
assert_divide("NaN", "-0.0");

trace("//\"-0.0\" / \"-0.0\"");
assert_divide("-0.0", "-0.0");

trace("//\"Infinity\" / \"-0.0\"");
assert_divide("Infinity", "-0.0");

trace("//\"1.0\" / \"-0.0\"");
assert_divide("1.0", "-0.0");

trace("//\"-1.0\" / \"-0.0\"");
assert_divide("-1.0", "-0.0");

trace("//\"0xFF1306\" / \"-0.0\"");
assert_divide("0xFF1306", "-0.0");

trace("//true / \"Infinity\"");
assert_divide(true, "Infinity");

trace("//false / \"Infinity\"");
assert_divide(false, "Infinity");

trace("//null / \"Infinity\"");
assert_divide(null, "Infinity");

trace("//undefined / \"Infinity\"");
assert_divide(undefined, "Infinity");

trace("//\"\" / \"Infinity\"");
assert_divide("", "Infinity");

trace("//\"str\" / \"Infinity\"");
assert_divide("str", "Infinity");

trace("//\"true\" / \"Infinity\"");
assert_divide("true", "Infinity");

trace("//\"false\" / \"Infinity\"");
assert_divide("false", "Infinity");

trace("//0.0 / \"Infinity\"");
assert_divide(0.0, "Infinity");

trace("//NaN / \"Infinity\"");
assert_divide(NaN, "Infinity");

trace("//-0.0 / \"Infinity\"");
assert_divide(-0.0, "Infinity");

trace("//Infinity / \"Infinity\"");
assert_divide(Infinity, "Infinity");

trace("//1.0 / \"Infinity\"");
assert_divide(1.0, "Infinity");

trace("//-1.0 / \"Infinity\"");
assert_divide(-1.0, "Infinity");

trace("//0xFF1306 / \"Infinity\"");
assert_divide(0xFF1306, "Infinity");

trace("//new Object() / \"Infinity\"");
assert_divide({}, "Infinity");

trace("//\"0.0\" / \"Infinity\"");
assert_divide("0.0", "Infinity");

trace("//\"NaN\" / \"Infinity\"");
assert_divide("NaN", "Infinity");

trace("//\"-0.0\" / \"Infinity\"");
assert_divide("-0.0", "Infinity");

trace("//\"Infinity\" / \"Infinity\"");
assert_divide("Infinity", "Infinity");

trace("//\"1.0\" / \"Infinity\"");
assert_divide("1.0", "Infinity");

trace("//\"-1.0\" / \"Infinity\"");
assert_divide("-1.0", "Infinity");

trace("//\"0xFF1306\" / \"Infinity\"");
assert_divide("0xFF1306", "Infinity");

trace("//true / \"1.0\"");
assert_divide(true, "1.0");

trace("//false / \"1.0\"");
assert_divide(false, "1.0");

trace("//null / \"1.0\"");
assert_divide(null, "1.0");

trace("//undefined / \"1.0\"");
assert_divide(undefined, "1.0");

trace("//\"\" / \"1.0\"");
assert_divide("", "1.0");

trace("//\"str\" / \"1.0\"");
assert_divide("str", "1.0");

trace("//\"true\" / \"1.0\"");
assert_divide("true", "1.0");

trace("//\"false\" / \"1.0\"");
assert_divide("false", "1.0");

trace("//0.0 / \"1.0\"");
assert_divide(0.0, "1.0");

trace("//NaN / \"1.0\"");
assert_divide(NaN, "1.0");

trace("//-0.0 / \"1.0\"");
assert_divide(-0.0, "1.0");

trace("//Infinity / \"1.0\"");
assert_divide(Infinity, "1.0");

trace("//1.0 / \"1.0\"");
assert_divide(1.0, "1.0");

trace("//-1.0 / \"1.0\"");
assert_divide(-1.0, "1.0");

trace("//0xFF1306 / \"1.0\"");
assert_divide(0xFF1306, "1.0");

trace("//new Object() / \"1.0\"");
assert_divide({}, "1.0");

trace("//\"0.0\" / \"1.0\"");
assert_divide("0.0", "1.0");

trace("//\"NaN\" / \"1.0\"");
assert_divide("NaN", "1.0");

trace("//\"-0.0\" / \"1.0\"");
assert_divide("-0.0", "1.0");

trace("//\"Infinity\" / \"1.0\"");
assert_divide("Infinity", "1.0");

trace("//\"1.0\" / \"1.0\"");
assert_divide("1.0", "1.0");

trace("//\"-1.0\" / \"1.0\"");
assert_divide("-1.0", "1.0");

trace("//\"0xFF1306\" / \"1.0\"");
assert_divide("0xFF1306", "1.0");

trace("//true / \"-1.0\"");
assert_divide(true, "-1.0");

trace("//false / \"-1.0\"");
assert_divide(false, "-1.0");

trace("//null / \"-1.0\"");
assert_divide(null, "-1.0");

trace("//undefined / \"-1.0\"");
assert_divide(undefined, "-1.0");

trace("//\"\" / \"-1.0\"");
assert_divide("", "-1.0");

trace("//\"str\" / \"-1.0\"");
assert_divide("str", "-1.0");

trace("//\"true\" / \"-1.0\"");
assert_divide("true", "-1.0");

trace("//\"false\" / \"-1.0\"");
assert_divide("false", "-1.0");

trace("//0.0 / \"-1.0\"");
assert_divide(0.0, "-1.0");

trace("//NaN / \"-1.0\"");
assert_divide(NaN, "-1.0");

trace("//-0.0 / \"-1.0\"");
assert_divide(-0.0, "-1.0");

trace("//Infinity / \"-1.0\"");
assert_divide(Infinity, "-1.0");

trace("//1.0 / \"-1.0\"");
assert_divide(1.0, "-1.0");

trace("//-1.0 / \"-1.0\"");
assert_divide(-1.0, "-1.0");

trace("//0xFF1306 / \"-1.0\"");
assert_divide(0xFF1306, "-1.0");

trace("//new Object() / \"-1.0\"");
assert_divide({}, "-1.0");

trace("//\"0.0\" / \"-1.0\"");
assert_divide("0.0", "-1.0");

trace("//\"NaN\" / \"-1.0\"");
assert_divide("NaN", "-1.0");

trace("//\"-0.0\" / \"-1.0\"");
assert_divide("-0.0", "-1.0");

trace("//\"Infinity\" / \"-1.0\"");
assert_divide("Infinity", "-1.0");

trace("//\"1.0\" / \"-1.0\"");
assert_divide("1.0", "-1.0");

trace("//\"-1.0\" / \"-1.0\"");
assert_divide("-1.0", "-1.0");

trace("//\"0xFF1306\" / \"-1.0\"");
assert_divide("0xFF1306", "-1.0");

trace("//true / \"0xFF1306\"");
assert_divide(true, "0xFF1306");

trace("//false / \"0xFF1306\"");
assert_divide(false, "0xFF1306");

trace("//null / \"0xFF1306\"");
assert_divide(null, "0xFF1306");

trace("//undefined / \"0xFF1306\"");
assert_divide(undefined, "0xFF1306");

trace("//\"\" / \"0xFF1306\"");
assert_divide("", "0xFF1306");

trace("//\"str\" / \"0xFF1306\"");
assert_divide("str", "0xFF1306");

trace("//\"true\" / \"0xFF1306\"");
assert_divide("true", "0xFF1306");

trace("//\"false\" / \"0xFF1306\"");
assert_divide("false", "0xFF1306");

trace("//0.0 / \"0xFF1306\"");
assert_divide(0.0, "0xFF1306");

trace("//NaN / \"0xFF1306\"");
assert_divide(NaN, "0xFF1306");

trace("//-0.0 / \"0xFF1306\"");
assert_divide(-0.0, "0xFF1306");

trace("//Infinity / \"0xFF1306\"");
assert_divide(Infinity, "0xFF1306");

trace("//1.0 / \"0xFF1306\"");
assert_divide(1.0, "0xFF1306");

trace("//-1.0 / \"0xFF1306\"");
assert_divide(-1.0, "0xFF1306");

trace("//0xFF1306 / \"0xFF1306\"");
assert_divide(0xFF1306, "0xFF1306");

trace("//new Object() / \"0xFF1306\"");
assert_divide({}, "0xFF1306");

trace("//\"0.0\" / \"0xFF1306\"");
assert_divide("0.0", "0xFF1306");

trace("//\"NaN\" / \"0xFF1306\"");
assert_divide("NaN", "0xFF1306");

trace("//\"-0.0\" / \"0xFF1306\"");
assert_divide("-0.0", "0xFF1306");

trace("//\"Infinity\" / \"0xFF1306\"");
assert_divide("Infinity", "0xFF1306");

trace("//\"1.0\" / \"0xFF1306\"");
assert_divide("1.0", "0xFF1306");

trace("//\"-1.0\" / \"0xFF1306\"");
assert_divide("-1.0", "0xFF1306");

trace("//\"0xFF1306\" / \"0xFF1306\"");
assert_divide("0xFF1306", "0xFF1306");