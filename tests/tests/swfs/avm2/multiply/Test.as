package {
	public class Test {
	}
}

function assert_multiply(val1, val2) {
	trace(val1 * val2);
}

trace("//true * true");
assert_multiply(true, true);

trace("//false * true");
assert_multiply(false, true);

trace("//null * true");
assert_multiply(null, true);

trace("//undefined * true");
assert_multiply(undefined, true);

trace("//\"\" * true");
assert_multiply("", true);

trace("//\"str\" * true");
assert_multiply("str", true);

trace("//\"true\" * true");
assert_multiply("true", true);

trace("//\"false\" * true");
assert_multiply("false", true);

trace("//0.0 * true");
assert_multiply(0.0, true);

trace("//NaN * true");
assert_multiply(NaN, true);

trace("//-0.0 * true");
assert_multiply(-0.0, true);

trace("//Infinity * true");
assert_multiply(Infinity, true);

trace("//1.0 * true");
assert_multiply(1.0, true);

trace("//-1.0 * true");
assert_multiply(-1.0, true);

trace("//0xFF1306 * true");
assert_multiply(0xFF1306, true);

trace("//new Object() * true");
assert_multiply({}, true);

trace("//\"0.0\" * true");
assert_multiply("0.0", true);

trace("//\"NaN\" * true");
assert_multiply("NaN", true);

trace("//\"-0.0\" * true");
assert_multiply("-0.0", true);

trace("//\"Infinity\" * true");
assert_multiply("Infinity", true);

trace("//\"1.0\" * true");
assert_multiply("1.0", true);

trace("//\"-1.0\" * true");
assert_multiply("-1.0", true);

trace("//\"0xFF1306\" * true");
assert_multiply("0xFF1306", true);

trace("//true * false");
assert_multiply(true, false);

trace("//false * false");
assert_multiply(false, false);

trace("//null * false");
assert_multiply(null, false);

trace("//undefined * false");
assert_multiply(undefined, false);

trace("//\"\" * false");
assert_multiply("", false);

trace("//\"str\" * false");
assert_multiply("str", false);

trace("//\"true\" * false");
assert_multiply("true", false);

trace("//\"false\" * false");
assert_multiply("false", false);

trace("//0.0 * false");
assert_multiply(0.0, false);

trace("//NaN * false");
assert_multiply(NaN, false);

trace("//-0.0 * false");
assert_multiply(-0.0, false);

trace("//Infinity * false");
assert_multiply(Infinity, false);

trace("//1.0 * false");
assert_multiply(1.0, false);

trace("//-1.0 * false");
assert_multiply(-1.0, false);

trace("//0xFF1306 * false");
assert_multiply(0xFF1306, false);

trace("//new Object() * false");
assert_multiply({}, false);

trace("//\"0.0\" * false");
assert_multiply("0.0", false);

trace("//\"NaN\" * false");
assert_multiply("NaN", false);

trace("//\"-0.0\" * false");
assert_multiply("-0.0", false);

trace("//\"Infinity\" * false");
assert_multiply("Infinity", false);

trace("//\"1.0\" * false");
assert_multiply("1.0", false);

trace("//\"-1.0\" * false");
assert_multiply("-1.0", false);

trace("//\"0xFF1306\" * false");
assert_multiply("0xFF1306", false);
trace("//true * null");
assert_multiply(true, null);

trace("//false * null");
assert_multiply(false, null);

trace("//null * null");
assert_multiply(null, null);

trace("//undefined * null");
assert_multiply(undefined, null);

trace("//\"\" * null");
assert_multiply("", null);

trace("//\"str\" * null");
assert_multiply("str", null);

trace("//\"true\" * null");
assert_multiply("true", null);

trace("//\"false\" * null");
assert_multiply("false", null);

trace("//0.0 * null");
assert_multiply(0.0, null);

trace("//NaN * null");
assert_multiply(NaN, null);

trace("//-0.0 * null");
assert_multiply(-0.0, null);

trace("//Infinity * null");
assert_multiply(Infinity, null);

trace("//1.0 * null");
assert_multiply(1.0, null);

trace("//-1.0 * null");
assert_multiply(-1.0, null);

trace("//0xFF1306 * null");
assert_multiply(0xFF1306, null);

trace("//new Object() * null");
assert_multiply({}, null);

trace("//\"0.0\" * null");
assert_multiply("0.0", null);

trace("//\"NaN\" * null");
assert_multiply("NaN", null);

trace("//\"-0.0\" * null");
assert_multiply("-0.0", null);

trace("//\"Infinity\" * null");
assert_multiply("Infinity", null);

trace("//\"1.0\" * null");
assert_multiply("1.0", null);

trace("//\"-1.0\" * null");
assert_multiply("-1.0", null);

trace("//\"0xFF1306\" * null");
assert_multiply("0xFF1306", null);

trace("//true * undefined");
assert_multiply(true, undefined);

trace("//false * undefined");
assert_multiply(false, undefined);

trace("//null * undefined");
assert_multiply(null, undefined);

trace("//undefined * undefined");
assert_multiply(undefined, undefined);

trace("//\"\" * undefined");
assert_multiply("", undefined);

trace("//\"str\" * undefined");
assert_multiply("str", undefined);

trace("//\"true\" * undefined");
assert_multiply("true", undefined);

trace("//\"false\" * undefined");
assert_multiply("false", undefined);

trace("//0.0 * undefined");
assert_multiply(0.0, undefined);

trace("//NaN * undefined");
assert_multiply(NaN, undefined);

trace("//-0.0 * undefined");
assert_multiply(-0.0, undefined);

trace("//Infinity * undefined");
assert_multiply(Infinity, undefined);

trace("//1.0 * undefined");
assert_multiply(1.0, undefined);

trace("//-1.0 * undefined");
assert_multiply(-1.0, undefined);

trace("//0xFF1306 * undefined");
assert_multiply(0xFF1306, undefined);

trace("//new Object() * undefined");
assert_multiply({}, undefined);

trace("//\"0.0\" * undefined");
assert_multiply("0.0", undefined);

trace("//\"NaN\" * undefined");
assert_multiply("NaN", undefined);

trace("//\"-0.0\" * undefined");
assert_multiply("-0.0", undefined);

trace("//\"Infinity\" * undefined");
assert_multiply("Infinity", undefined);

trace("//\"1.0\" * undefined");
assert_multiply("1.0", undefined);

trace("//\"-1.0\" * undefined");
assert_multiply("-1.0", undefined);

trace("//\"0xFF1306\" * undefined");
assert_multiply("0xFF1306", undefined);

trace("//true * \"\"");
assert_multiply(true, "");

trace("//false * \"\"");
assert_multiply(false, "");

trace("//null * \"\"");
assert_multiply(null, "");

trace("//undefined * \"\"");
assert_multiply(undefined, "");

trace("//\"\" * \"\"");
assert_multiply("", "");

trace("//\"str\" * \"\"");
assert_multiply("str", "");

trace("//\"true\" * \"\"");
assert_multiply("true", "");

trace("//\"false\" * \"\"");
assert_multiply("false", "");

trace("//0.0 * \"\"");
assert_multiply(0.0, "");

trace("//NaN * \"\"");
assert_multiply(NaN, "");

trace("//-0.0 * \"\"");
assert_multiply(-0.0, "");

trace("//Infinity * \"\"");
assert_multiply(Infinity, "");

trace("//1.0 * \"\"");
assert_multiply(1.0, "");

trace("//-1.0 * \"\"");
assert_multiply(-1.0, "");

trace("//0xFF1306 * \"\"");
assert_multiply(0xFF1306, "");

trace("//new Object() * \"\"");
assert_multiply({}, "");

trace("//\"0.0\" * \"\"");
assert_multiply("0.0", "");

trace("//\"NaN\" * \"\"");
assert_multiply("NaN", "");

trace("//\"-0.0\" * \"\"");
assert_multiply("-0.0", "");

trace("//\"Infinity\" * \"\"");
assert_multiply("Infinity", "");

trace("//\"1.0\" * \"\"");
assert_multiply("1.0", "");

trace("//\"-1.0\" * \"\"");
assert_multiply("-1.0", "");

trace("//\"0xFF1306\" * \"\"");
assert_multiply("0xFF1306", "");

trace("//true * \"str\"");
assert_multiply(true, "str");

trace("//false * \"str\"");
assert_multiply(false, "str");

trace("//null * \"str\"");
assert_multiply(null, "str");

trace("//undefined * \"str\"");
assert_multiply(undefined, "str");

trace("//\"\" * \"str\"");
assert_multiply("", "str");

trace("//\"str\" * \"str\"");
assert_multiply("str", "str");

trace("//\"true\" * \"str\"");
assert_multiply("true", "str");

trace("//\"false\" * \"str\"");
assert_multiply("false", "str");

trace("//0.0 * \"str\"");
assert_multiply(0.0, "str");

trace("//NaN * \"str\"");
assert_multiply(NaN, "str");

trace("//-0.0 * \"str\"");
assert_multiply(-0.0, "str");

trace("//Infinity * \"str\"");
assert_multiply(Infinity, "str");

trace("//1.0 * \"str\"");
assert_multiply(1.0, "str");

trace("//-1.0 * \"str\"");
assert_multiply(-1.0, "str");

trace("//0xFF1306 * \"str\"");
assert_multiply(0xFF1306, "str");

trace("//new Object() * \"str\"");
assert_multiply({}, "str");

trace("//\"0.0\" * \"str\"");
assert_multiply("0.0", "str");

trace("//\"NaN\" * \"str\"");
assert_multiply("NaN", "str");

trace("//\"-0.0\" * \"str\"");
assert_multiply("-0.0", "str");

trace("//\"Infinity\" * \"str\"");
assert_multiply("Infinity", "str");

trace("//\"1.0\" * \"str\"");
assert_multiply("1.0", "str");

trace("//\"-1.0\" * \"str\"");
assert_multiply("-1.0", "str");

trace("//\"0xFF1306\" * \"str\"");
assert_multiply("0xFF1306", "str");

trace("//true * \"true\"");
assert_multiply(true, "true");

trace("//false * \"true\"");
assert_multiply(false, "true");

trace("//null * \"true\"");
assert_multiply(null, "true");

trace("//undefined * \"true\"");
assert_multiply(undefined, "true");

trace("//\"\" * \"true\"");
assert_multiply("", "true");

trace("//\"str\" * \"true\"");
assert_multiply("str", "true");

trace("//\"true\" * \"true\"");
assert_multiply("true", "true");

trace("//\"false\" * \"true\"");
assert_multiply("false", "true");

trace("//0.0 * \"true\"");
assert_multiply(0.0, "true");

trace("//NaN * \"true\"");
assert_multiply(NaN, "true");

trace("//-0.0 * \"true\"");
assert_multiply(-0.0, "true");

trace("//Infinity * \"true\"");
assert_multiply(Infinity, "true");

trace("//1.0 * \"true\"");
assert_multiply(1.0, "true");

trace("//-1.0 * \"true\"");
assert_multiply(-1.0, "true");

trace("//0xFF1306 * \"true\"");
assert_multiply(0xFF1306, "true");

trace("//new Object() * \"true\"");
assert_multiply({}, "true");

trace("//\"0.0\" * \"true\"");
assert_multiply("0.0", "true");

trace("//\"NaN\" * \"true\"");
assert_multiply("NaN", "true");

trace("//\"-0.0\" * \"true\"");
assert_multiply("-0.0", "true");

trace("//\"Infinity\" * \"true\"");
assert_multiply("Infinity", "true");

trace("//\"1.0\" * \"true\"");
assert_multiply("1.0", "true");

trace("//\"-1.0\" * \"true\"");
assert_multiply("-1.0", "true");

trace("//\"0xFF1306\" * \"true\"");
assert_multiply("0xFF1306", "true");

trace("//true * \"false\"");
assert_multiply(true, "false");

trace("//false * \"false\"");
assert_multiply(false, "false");

trace("//null * \"false\"");
assert_multiply(null, "false");

trace("//undefined * \"false\"");
assert_multiply(undefined, "false");

trace("//\"\" * \"false\"");
assert_multiply("", "false");

trace("//\"str\" * \"false\"");
assert_multiply("str", "false");

trace("//\"true\" * \"false\"");
assert_multiply("true", "false");

trace("//\"false\" * \"false\"");
assert_multiply("false", "false");

trace("//0.0 * \"false\"");
assert_multiply(0.0, "false");

trace("//NaN * \"false\"");
assert_multiply(NaN, "false");

trace("//-0.0 * \"false\"");
assert_multiply(-0.0, "false");

trace("//Infinity * \"false\"");
assert_multiply(Infinity, "false");

trace("//1.0 * \"false\"");
assert_multiply(1.0, "false");

trace("//-1.0 * \"false\"");
assert_multiply(-1.0, "false");

trace("//0xFF1306 * \"false\"");
assert_multiply(0xFF1306, "false");

trace("//new Object() * \"false\"");
assert_multiply({}, "false");

trace("//\"0.0\" * \"false\"");
assert_multiply("0.0", "false");

trace("//\"NaN\" * \"false\"");
assert_multiply("NaN", "false");

trace("//\"-0.0\" * \"false\"");
assert_multiply("-0.0", "false");

trace("//\"Infinity\" * \"false\"");
assert_multiply("Infinity", "false");

trace("//\"1.0\" * \"false\"");
assert_multiply("1.0", "false");

trace("//\"-1.0\" * \"false\"");
assert_multiply("-1.0", "false");

trace("//\"0xFF1306\" * \"false\"");
assert_multiply("0xFF1306", "false");

trace("//true * 0.0");
assert_multiply(true, 0.0);

trace("//false * 0.0");
assert_multiply(false, 0.0);

trace("//null * 0.0");
assert_multiply(null, 0.0);

trace("//undefined * 0.0");
assert_multiply(undefined, 0.0);

trace("//\"\" * 0.0");
assert_multiply("", 0.0);

trace("//\"str\" * 0.0");
assert_multiply("str", 0.0);

trace("//\"true\" * 0.0");
assert_multiply("true", 0.0);

trace("//\"false\" * 0.0");
assert_multiply("false", 0.0);

trace("//0.0 * 0.0");
assert_multiply(0.0, 0.0);

trace("//NaN * 0.0");
assert_multiply(NaN, 0.0);

trace("//-0.0 * 0.0");
assert_multiply(-0.0, 0.0);

trace("//Infinity * 0.0");
assert_multiply(Infinity, 0.0);

trace("//1.0 * 0.0");
assert_multiply(1.0, 0.0);

trace("//-1.0 * 0.0");
assert_multiply(-1.0, 0.0);

trace("//0xFF1306 * 0.0");
assert_multiply(0xFF1306, 0.0);

trace("//new Object() * 0.0");
assert_multiply({}, 0.0);

trace("//\"0.0\" * 0.0");
assert_multiply("0.0", 0.0);

trace("//\"NaN\" * 0.0");
assert_multiply("NaN", 0.0);

trace("//\"-0.0\" * 0.0");
assert_multiply("-0.0", 0.0);

trace("//\"Infinity\" * 0.0");
assert_multiply("Infinity", 0.0);

trace("//\"1.0\" * 0.0");
assert_multiply("1.0", 0.0);

trace("//\"-1.0\" * 0.0");
assert_multiply("-1.0", 0.0);

trace("//\"0xFF1306\" * 0.0");
assert_multiply("0xFF1306", 0.0);

trace("//true * NaN");
assert_multiply(true, NaN);

trace("//false * NaN");
assert_multiply(false, NaN);

trace("//null * NaN");
assert_multiply(null, NaN);

trace("//undefined * NaN");
assert_multiply(undefined, NaN);

trace("//\"\" * NaN");
assert_multiply("", NaN);

trace("//\"str\" * NaN");
assert_multiply("str", NaN);

trace("//\"true\" * NaN");
assert_multiply("true", NaN);

trace("//\"false\" * NaN");
assert_multiply("false", NaN);

trace("//0.0 * NaN");
assert_multiply(0.0, NaN);

trace("//NaN * NaN");
assert_multiply(NaN, NaN);

trace("//-0.0 * NaN");
assert_multiply(-0.0, NaN);

trace("//Infinity * NaN");
assert_multiply(Infinity, NaN);

trace("//1.0 * NaN");
assert_multiply(1.0, NaN);

trace("//-1.0 * NaN");
assert_multiply(-1.0, NaN);

trace("//0xFF1306 * NaN");
assert_multiply(0xFF1306, NaN);

trace("//new Object() * NaN");
assert_multiply({}, NaN);

trace("//\"0.0\" * NaN");
assert_multiply("0.0", NaN);

trace("//\"NaN\" * NaN");
assert_multiply("NaN", NaN);

trace("//\"-0.0\" * NaN");
assert_multiply("-0.0", NaN);

trace("//\"Infinity\" * NaN");
assert_multiply("Infinity", NaN);

trace("//\"1.0\" * NaN");
assert_multiply("1.0", NaN);

trace("//\"-1.0\" * NaN");
assert_multiply("-1.0", NaN);

trace("//\"0xFF1306\" * NaN");
assert_multiply("0xFF1306", NaN);

trace("//true * -0.0");
assert_multiply(true, -0.0);

trace("//false * -0.0");
assert_multiply(false, -0.0);

trace("//null * -0.0");
assert_multiply(null, -0.0);

trace("//undefined * -0.0");
assert_multiply(undefined, -0.0);

trace("//\"\" * -0.0");
assert_multiply("", -0.0);

trace("//\"str\" * -0.0");
assert_multiply("str", -0.0);

trace("//\"true\" * -0.0");
assert_multiply("true", -0.0);

trace("//\"false\" * -0.0");
assert_multiply("false", -0.0);

trace("//0.0 * -0.0");
assert_multiply(0.0, -0.0);

trace("//NaN * -0.0");
assert_multiply(NaN, -0.0);

trace("//-0.0 * -0.0");
assert_multiply(-0.0, -0.0);

trace("//Infinity * -0.0");
assert_multiply(Infinity, -0.0);

trace("//1.0 * -0.0");
assert_multiply(1.0, -0.0);

trace("//-1.0 * -0.0");
assert_multiply(-1.0, -0.0);

trace("//0xFF1306 * -0.0");
assert_multiply(0xFF1306, -0.0);

trace("//new Object() * -0.0");
assert_multiply({}, -0.0);

trace("//\"0.0\" * -0.0");
assert_multiply("0.0", -0.0);

trace("//\"NaN\" * -0.0");
assert_multiply("NaN", -0.0);

trace("//\"-0.0\" * -0.0");
assert_multiply("-0.0", -0.0);

trace("//\"Infinity\" * -0.0");
assert_multiply("Infinity", -0.0);

trace("//\"1.0\" * -0.0");
assert_multiply("1.0", -0.0);

trace("//\"-1.0\" * -0.0");
assert_multiply("-1.0", -0.0);

trace("//\"0xFF1306\" * -0.0");
assert_multiply("0xFF1306", -0.0);

trace("//true * Infinity");
assert_multiply(true, Infinity);

trace("//false * Infinity");
assert_multiply(false, Infinity);

trace("//null * Infinity");
assert_multiply(null, Infinity);

trace("//undefined * Infinity");
assert_multiply(undefined, Infinity);

trace("//\"\" * Infinity");
assert_multiply("", Infinity);

trace("//\"str\" * Infinity");
assert_multiply("str", Infinity);

trace("//\"true\" * Infinity");
assert_multiply("true", Infinity);

trace("//\"false\" * Infinity");
assert_multiply("false", Infinity);

trace("//0.0 * Infinity");
assert_multiply(0.0, Infinity);

trace("//NaN * Infinity");
assert_multiply(NaN, Infinity);

trace("//-0.0 * Infinity");
assert_multiply(-0.0, Infinity);

trace("//Infinity * Infinity");
assert_multiply(Infinity, Infinity);

trace("//1.0 * Infinity");
assert_multiply(1.0, Infinity);

trace("//-1.0 * Infinity");
assert_multiply(-1.0, Infinity);

trace("//0xFF1306 * Infinity");
assert_multiply(0xFF1306, Infinity);

trace("//new Object() * Infinity");
assert_multiply({}, Infinity);

trace("//\"0.0\" * Infinity");
assert_multiply("0.0", Infinity);

trace("//\"NaN\" * Infinity");
assert_multiply("NaN", Infinity);

trace("//\"-0.0\" * Infinity");
assert_multiply("-0.0", Infinity);

trace("//\"Infinity\" * Infinity");
assert_multiply("Infinity", Infinity);

trace("//\"1.0\" * Infinity");
assert_multiply("1.0", Infinity);

trace("//\"-1.0\" * Infinity");
assert_multiply("-1.0", Infinity);

trace("//\"0xFF1306\" * Infinity");
assert_multiply("0xFF1306", Infinity);

trace("//true * 1.0");
assert_multiply(true, 1.0);

trace("//false * 1.0");
assert_multiply(false, 1.0);

trace("//null * 1.0");
assert_multiply(null, 1.0);

trace("//undefined * 1.0");
assert_multiply(undefined, 1.0);

trace("//\"\" * 1.0");
assert_multiply("", 1.0);

trace("//\"str\" * 1.0");
assert_multiply("str", 1.0);

trace("//\"true\" * 1.0");
assert_multiply("true", 1.0);

trace("//\"false\" * 1.0");
assert_multiply("false", 1.0);

trace("//0.0 * 1.0");
assert_multiply(0.0, 1.0);

trace("//NaN * 1.0");
assert_multiply(NaN, 1.0);

trace("//-0.0 * 1.0");
assert_multiply(-0.0, 1.0);

trace("//Infinity * 1.0");
assert_multiply(Infinity, 1.0);

trace("//1.0 * 1.0");
assert_multiply(1.0, 1.0);

trace("//-1.0 * 1.0");
assert_multiply(-1.0, 1.0);

trace("//0xFF1306 * 1.0");
assert_multiply(0xFF1306, 1.0);

trace("//new Object() * 1.0");
assert_multiply({}, 1.0);

trace("//\"0.0\" * 1.0");
assert_multiply("0.0", 1.0);

trace("//\"NaN\" * 1.0");
assert_multiply("NaN", 1.0);

trace("//\"-0.0\" * 1.0");
assert_multiply("-0.0", 1.0);

trace("//\"Infinity\" * 1.0");
assert_multiply("Infinity", 1.0);

trace("//\"1.0\" * 1.0");
assert_multiply("1.0", 1.0);

trace("//\"-1.0\" * 1.0");
assert_multiply("-1.0", 1.0);

trace("//\"0xFF1306\" * 1.0");
assert_multiply("0xFF1306", 1.0);

trace("//true * -1.0");
assert_multiply(true, -1.0);

trace("//false * -1.0");
assert_multiply(false, -1.0);

trace("//null * -1.0");
assert_multiply(null, -1.0);

trace("//undefined * -1.0");
assert_multiply(undefined, -1.0);

trace("//\"\" * -1.0");
assert_multiply("", -1.0);

trace("//\"str\" * -1.0");
assert_multiply("str", -1.0);

trace("//\"true\" * -1.0");
assert_multiply("true", -1.0);

trace("//\"false\" * -1.0");
assert_multiply("false", -1.0);

trace("//0.0 * -1.0");
assert_multiply(0.0, -1.0);

trace("//NaN * -1.0");
assert_multiply(NaN, -1.0);

trace("//-0.0 * -1.0");
assert_multiply(-0.0, -1.0);

trace("//Infinity * -1.0");
assert_multiply(Infinity, -1.0);

trace("//1.0 * -1.0");
assert_multiply(1.0, -1.0);

trace("//-1.0 * -1.0");
assert_multiply(-1.0, -1.0);

trace("//0xFF1306 * -1.0");
assert_multiply(0xFF1306, -1.0);

trace("//new Object() * -1.0");
assert_multiply({}, -1.0);

trace("//\"0.0\" * -1.0");
assert_multiply("0.0", -1.0);

trace("//\"NaN\" * -1.0");
assert_multiply("NaN", -1.0);

trace("//\"-0.0\" * -1.0");
assert_multiply("-0.0", -1.0);

trace("//\"Infinity\" * -1.0");
assert_multiply("Infinity", -1.0);

trace("//\"1.0\" * -1.0");
assert_multiply("1.0", -1.0);

trace("//\"-1.0\" * -1.0");
assert_multiply("-1.0", -1.0);

trace("//\"0xFF1306\" * -1.0");
assert_multiply("0xFF1306", -1.0);

trace("//true * 0xFF1306");
assert_multiply(true, 0xFF1306);

trace("//false * 0xFF1306");
assert_multiply(false, 0xFF1306);

trace("//null * 0xFF1306");
assert_multiply(null, 0xFF1306);

trace("//undefined * 0xFF1306");
assert_multiply(undefined, 0xFF1306);

trace("//\"\" * 0xFF1306");
assert_multiply("", 0xFF1306);

trace("//\"str\" * 0xFF1306");
assert_multiply("str", 0xFF1306);

trace("//\"true\" * 0xFF1306");
assert_multiply("true", 0xFF1306);

trace("//\"false\" * 0xFF1306");
assert_multiply("false", 0xFF1306);

trace("//0.0 * 0xFF1306");
assert_multiply(0.0, 0xFF1306);

trace("//NaN * 0xFF1306");
assert_multiply(NaN, 0xFF1306);

trace("//-0.0 * 0xFF1306");
assert_multiply(-0.0, 0xFF1306);

trace("//Infinity * 0xFF1306");
assert_multiply(Infinity, 0xFF1306);

trace("//1.0 * 0xFF1306");
assert_multiply(1.0, 0xFF1306);

trace("//-1.0 * 0xFF1306");
assert_multiply(-1.0, 0xFF1306);

trace("//0xFF1306 * 0xFF1306");
assert_multiply(0xFF1306, 0xFF1306);

trace("//new Object() * 0xFF1306");
assert_multiply({}, 0xFF1306);

trace("//\"0.0\" * 0xFF1306");
assert_multiply("0.0", 0xFF1306);

trace("//\"NaN\" * 0xFF1306");
assert_multiply("NaN", 0xFF1306);

trace("//\"-0.0\" * 0xFF1306");
assert_multiply("-0.0", 0xFF1306);

trace("//\"Infinity\" * 0xFF1306");
assert_multiply("Infinity", 0xFF1306);

trace("//\"1.0\" * 0xFF1306");
assert_multiply("1.0", 0xFF1306);

trace("//\"-1.0\" * 0xFF1306");
assert_multiply("-1.0", 0xFF1306);

trace("//\"0xFF1306\" * 0xFF1306");
assert_multiply("0xFF1306", 0xFF1306);

trace("//true * new Object()");
assert_multiply(true, {});

trace("//false * new Object()");
assert_multiply(false, {});

trace("//null * new Object()");
assert_multiply(null, {});

trace("//undefined * new Object()");
assert_multiply(undefined, {});

trace("//\"\" * new Object()");
assert_multiply("", {});

trace("//\"str\" * new Object()");
assert_multiply("str", {});

trace("//\"true\" * new Object()");
assert_multiply("true", {});

trace("//\"false\" * new Object()");
assert_multiply("false", {});

trace("//0.0 * new Object()");
assert_multiply(0.0, {});

trace("//NaN * new Object()");
assert_multiply(NaN, {});

trace("//-0.0 * new Object()");
assert_multiply(-0.0, {});

trace("//Infinity * new Object()");
assert_multiply(Infinity, {});

trace("//1.0 * new Object()");
assert_multiply(1.0, {});

trace("//-1.0 * new Object()");
assert_multiply(-1.0, {});

trace("//0xFF1306 * new Object()");
assert_multiply(0xFF1306, {});

trace("//new Object() * new Object()");
assert_multiply({}, {});

trace("//\"0.0\" * new Object()");
assert_multiply("0.0", {});

trace("//\"NaN\" * new Object()");
assert_multiply("NaN", {});

trace("//\"-0.0\" * new Object()");
assert_multiply("-0.0", {});

trace("//\"Infinity\" * new Object()");
assert_multiply("Infinity", {});

trace("//\"1.0\" * new Object()");
assert_multiply("1.0", {});

trace("//\"-1.0\" * new Object()");
assert_multiply("-1.0", {});

trace("//\"0xFF1306\" * new Object()");
assert_multiply("0xFF1306", {});

trace("//true * \"0.0\"");
assert_multiply(true, "0.0");

trace("//false * \"0.0\"");
assert_multiply(false, "0.0");

trace("//null * \"0.0\"");
assert_multiply(null, "0.0");

trace("//undefined * \"0.0\"");
assert_multiply(undefined, "0.0");

trace("//\"\" * \"0.0\"");
assert_multiply("", "0.0");

trace("//\"str\" * \"0.0\"");
assert_multiply("str", "0.0");

trace("//\"true\" * \"0.0\"");
assert_multiply("true", "0.0");

trace("//\"false\" * \"0.0\"");
assert_multiply("false", "0.0");

trace("//0.0 * \"0.0\"");
assert_multiply(0.0, "0.0");

trace("//NaN * \"0.0\"");
assert_multiply(NaN, "0.0");

trace("//-0.0 * \"0.0\"");
assert_multiply(-0.0, "0.0");

trace("//Infinity * \"0.0\"");
assert_multiply(Infinity, "0.0");

trace("//1.0 * \"0.0\"");
assert_multiply(1.0, "0.0");

trace("//-1.0 * \"0.0\"");
assert_multiply(-1.0, "0.0");

trace("//0xFF1306 * \"0.0\"");
assert_multiply(0xFF1306, "0.0");

trace("//new Object() * \"0.0\"");
assert_multiply({}, "0.0");

trace("//\"0.0\" * \"0.0\"");
assert_multiply("0.0", "0.0");

trace("//\"NaN\" * \"0.0\"");
assert_multiply("NaN", "0.0");

trace("//\"-0.0\" * \"0.0\"");
assert_multiply("-0.0", "0.0");

trace("//\"Infinity\" * \"0.0\"");
assert_multiply("Infinity", "0.0");

trace("//\"1.0\" * \"0.0\"");
assert_multiply("1.0", "0.0");

trace("//\"-1.0\" * \"0.0\"");
assert_multiply("-1.0", "0.0");

trace("//\"0xFF1306\" * \"0.0\"");
assert_multiply("0xFF1306", "0.0");

trace("//true * \"NaN\"");
assert_multiply(true, "NaN");

trace("//false * \"NaN\"");
assert_multiply(false, "NaN");

trace("//null * \"NaN\"");
assert_multiply(null, "NaN");

trace("//undefined * \"NaN\"");
assert_multiply(undefined, "NaN");

trace("//\"\" * \"NaN\"");
assert_multiply("", "NaN");

trace("//\"str\" * \"NaN\"");
assert_multiply("str", "NaN");

trace("//\"true\" * \"NaN\"");
assert_multiply("true", "NaN");

trace("//\"false\" * \"NaN\"");
assert_multiply("false", "NaN");

trace("//0.0 * \"NaN\"");
assert_multiply(0.0, "NaN");

trace("//NaN * \"NaN\"");
assert_multiply(NaN, "NaN");

trace("//-0.0 * \"NaN\"");
assert_multiply(-0.0, "NaN");

trace("//Infinity * \"NaN\"");
assert_multiply(Infinity, "NaN");

trace("//1.0 * \"NaN\"");
assert_multiply(1.0, "NaN");

trace("//-1.0 * \"NaN\"");
assert_multiply(-1.0, "NaN");

trace("//0xFF1306 * \"NaN\"");
assert_multiply(0xFF1306, "NaN");

trace("//new Object() * \"NaN\"");
assert_multiply({}, "NaN");

trace("//\"0.0\" * \"NaN\"");
assert_multiply("0.0", "NaN");

trace("//\"NaN\" * \"NaN\"");
assert_multiply("NaN", "NaN");

trace("//\"-0.0\" * \"NaN\"");
assert_multiply("-0.0", "NaN");

trace("//\"Infinity\" * \"NaN\"");
assert_multiply("Infinity", "NaN");

trace("//\"1.0\" * \"NaN\"");
assert_multiply("1.0", "NaN");

trace("//\"-1.0\" * \"NaN\"");
assert_multiply("-1.0", "NaN");

trace("//\"0xFF1306\" * \"NaN\"");
assert_multiply("0xFF1306", "NaN");

trace("//true * \"-0.0\"");
assert_multiply(true, "-0.0");

trace("//false * \"-0.0\"");
assert_multiply(false, "-0.0");

trace("//null * \"-0.0\"");
assert_multiply(null, "-0.0");

trace("//undefined * \"-0.0\"");
assert_multiply(undefined, "-0.0");

trace("//\"\" * \"-0.0\"");
assert_multiply("", "-0.0");

trace("//\"str\" * \"-0.0\"");
assert_multiply("str", "-0.0");

trace("//\"true\" * \"-0.0\"");
assert_multiply("true", "-0.0");

trace("//\"false\" * \"-0.0\"");
assert_multiply("false", "-0.0");

trace("//0.0 * \"-0.0\"");
assert_multiply(0.0, "-0.0");

trace("//NaN * \"-0.0\"");
assert_multiply(NaN, "-0.0");

trace("//-0.0 * \"-0.0\"");
assert_multiply(-0.0, "-0.0");

trace("//Infinity * \"-0.0\"");
assert_multiply(Infinity, "-0.0");

trace("//1.0 * \"-0.0\"");
assert_multiply(1.0, "-0.0");

trace("//-1.0 * \"-0.0\"");
assert_multiply(-1.0, "-0.0");

trace("//0xFF1306 * \"-0.0\"");
assert_multiply(0xFF1306, "-0.0");

trace("//new Object() * \"-0.0\"");
assert_multiply({}, "-0.0");

trace("//\"0.0\" * \"-0.0\"");
assert_multiply("0.0", "-0.0");

trace("//\"NaN\" * \"-0.0\"");
assert_multiply("NaN", "-0.0");

trace("//\"-0.0\" * \"-0.0\"");
assert_multiply("-0.0", "-0.0");

trace("//\"Infinity\" * \"-0.0\"");
assert_multiply("Infinity", "-0.0");

trace("//\"1.0\" * \"-0.0\"");
assert_multiply("1.0", "-0.0");

trace("//\"-1.0\" * \"-0.0\"");
assert_multiply("-1.0", "-0.0");

trace("//\"0xFF1306\" * \"-0.0\"");
assert_multiply("0xFF1306", "-0.0");

trace("//true * \"Infinity\"");
assert_multiply(true, "Infinity");

trace("//false * \"Infinity\"");
assert_multiply(false, "Infinity");

trace("//null * \"Infinity\"");
assert_multiply(null, "Infinity");

trace("//undefined * \"Infinity\"");
assert_multiply(undefined, "Infinity");

trace("//\"\" * \"Infinity\"");
assert_multiply("", "Infinity");

trace("//\"str\" * \"Infinity\"");
assert_multiply("str", "Infinity");

trace("//\"true\" * \"Infinity\"");
assert_multiply("true", "Infinity");

trace("//\"false\" * \"Infinity\"");
assert_multiply("false", "Infinity");

trace("//0.0 * \"Infinity\"");
assert_multiply(0.0, "Infinity");

trace("//NaN * \"Infinity\"");
assert_multiply(NaN, "Infinity");

trace("//-0.0 * \"Infinity\"");
assert_multiply(-0.0, "Infinity");

trace("//Infinity * \"Infinity\"");
assert_multiply(Infinity, "Infinity");

trace("//1.0 * \"Infinity\"");
assert_multiply(1.0, "Infinity");

trace("//-1.0 * \"Infinity\"");
assert_multiply(-1.0, "Infinity");

trace("//0xFF1306 * \"Infinity\"");
assert_multiply(0xFF1306, "Infinity");

trace("//new Object() * \"Infinity\"");
assert_multiply({}, "Infinity");

trace("//\"0.0\" * \"Infinity\"");
assert_multiply("0.0", "Infinity");

trace("//\"NaN\" * \"Infinity\"");
assert_multiply("NaN", "Infinity");

trace("//\"-0.0\" * \"Infinity\"");
assert_multiply("-0.0", "Infinity");

trace("//\"Infinity\" * \"Infinity\"");
assert_multiply("Infinity", "Infinity");

trace("//\"1.0\" * \"Infinity\"");
assert_multiply("1.0", "Infinity");

trace("//\"-1.0\" * \"Infinity\"");
assert_multiply("-1.0", "Infinity");

trace("//\"0xFF1306\" * \"Infinity\"");
assert_multiply("0xFF1306", "Infinity");

trace("//true * \"1.0\"");
assert_multiply(true, "1.0");

trace("//false * \"1.0\"");
assert_multiply(false, "1.0");

trace("//null * \"1.0\"");
assert_multiply(null, "1.0");

trace("//undefined * \"1.0\"");
assert_multiply(undefined, "1.0");

trace("//\"\" * \"1.0\"");
assert_multiply("", "1.0");

trace("//\"str\" * \"1.0\"");
assert_multiply("str", "1.0");

trace("//\"true\" * \"1.0\"");
assert_multiply("true", "1.0");

trace("//\"false\" * \"1.0\"");
assert_multiply("false", "1.0");

trace("//0.0 * \"1.0\"");
assert_multiply(0.0, "1.0");

trace("//NaN * \"1.0\"");
assert_multiply(NaN, "1.0");

trace("//-0.0 * \"1.0\"");
assert_multiply(-0.0, "1.0");

trace("//Infinity * \"1.0\"");
assert_multiply(Infinity, "1.0");

trace("//1.0 * \"1.0\"");
assert_multiply(1.0, "1.0");

trace("//-1.0 * \"1.0\"");
assert_multiply(-1.0, "1.0");

trace("//0xFF1306 * \"1.0\"");
assert_multiply(0xFF1306, "1.0");

trace("//new Object() * \"1.0\"");
assert_multiply({}, "1.0");

trace("//\"0.0\" * \"1.0\"");
assert_multiply("0.0", "1.0");

trace("//\"NaN\" * \"1.0\"");
assert_multiply("NaN", "1.0");

trace("//\"-0.0\" * \"1.0\"");
assert_multiply("-0.0", "1.0");

trace("//\"Infinity\" * \"1.0\"");
assert_multiply("Infinity", "1.0");

trace("//\"1.0\" * \"1.0\"");
assert_multiply("1.0", "1.0");

trace("//\"-1.0\" * \"1.0\"");
assert_multiply("-1.0", "1.0");

trace("//\"0xFF1306\" * \"1.0\"");
assert_multiply("0xFF1306", "1.0");

trace("//true * \"-1.0\"");
assert_multiply(true, "-1.0");

trace("//false * \"-1.0\"");
assert_multiply(false, "-1.0");

trace("//null * \"-1.0\"");
assert_multiply(null, "-1.0");

trace("//undefined * \"-1.0\"");
assert_multiply(undefined, "-1.0");

trace("//\"\" * \"-1.0\"");
assert_multiply("", "-1.0");

trace("//\"str\" * \"-1.0\"");
assert_multiply("str", "-1.0");

trace("//\"true\" * \"-1.0\"");
assert_multiply("true", "-1.0");

trace("//\"false\" * \"-1.0\"");
assert_multiply("false", "-1.0");

trace("//0.0 * \"-1.0\"");
assert_multiply(0.0, "-1.0");

trace("//NaN * \"-1.0\"");
assert_multiply(NaN, "-1.0");

trace("//-0.0 * \"-1.0\"");
assert_multiply(-0.0, "-1.0");

trace("//Infinity * \"-1.0\"");
assert_multiply(Infinity, "-1.0");

trace("//1.0 * \"-1.0\"");
assert_multiply(1.0, "-1.0");

trace("//-1.0 * \"-1.0\"");
assert_multiply(-1.0, "-1.0");

trace("//0xFF1306 * \"-1.0\"");
assert_multiply(0xFF1306, "-1.0");

trace("//new Object() * \"-1.0\"");
assert_multiply({}, "-1.0");

trace("//\"0.0\" * \"-1.0\"");
assert_multiply("0.0", "-1.0");

trace("//\"NaN\" * \"-1.0\"");
assert_multiply("NaN", "-1.0");

trace("//\"-0.0\" * \"-1.0\"");
assert_multiply("-0.0", "-1.0");

trace("//\"Infinity\" * \"-1.0\"");
assert_multiply("Infinity", "-1.0");

trace("//\"1.0\" * \"-1.0\"");
assert_multiply("1.0", "-1.0");

trace("//\"-1.0\" * \"-1.0\"");
assert_multiply("-1.0", "-1.0");

trace("//\"0xFF1306\" * \"-1.0\"");
assert_multiply("0xFF1306", "-1.0");

trace("//true * \"0xFF1306\"");
assert_multiply(true, "0xFF1306");

trace("//false * \"0xFF1306\"");
assert_multiply(false, "0xFF1306");

trace("//null * \"0xFF1306\"");
assert_multiply(null, "0xFF1306");

trace("//undefined * \"0xFF1306\"");
assert_multiply(undefined, "0xFF1306");

trace("//\"\" * \"0xFF1306\"");
assert_multiply("", "0xFF1306");

trace("//\"str\" * \"0xFF1306\"");
assert_multiply("str", "0xFF1306");

trace("//\"true\" * \"0xFF1306\"");
assert_multiply("true", "0xFF1306");

trace("//\"false\" * \"0xFF1306\"");
assert_multiply("false", "0xFF1306");

trace("//0.0 * \"0xFF1306\"");
assert_multiply(0.0, "0xFF1306");

trace("//NaN * \"0xFF1306\"");
assert_multiply(NaN, "0xFF1306");

trace("//-0.0 * \"0xFF1306\"");
assert_multiply(-0.0, "0xFF1306");

trace("//Infinity * \"0xFF1306\"");
assert_multiply(Infinity, "0xFF1306");

trace("//1.0 * \"0xFF1306\"");
assert_multiply(1.0, "0xFF1306");

trace("//-1.0 * \"0xFF1306\"");
assert_multiply(-1.0, "0xFF1306");

trace("//0xFF1306 * \"0xFF1306\"");
assert_multiply(0xFF1306, "0xFF1306");

trace("//new Object() * \"0xFF1306\"");
assert_multiply({}, "0xFF1306");

trace("//\"0.0\" * \"0xFF1306\"");
assert_multiply("0.0", "0xFF1306");

trace("//\"NaN\" * \"0xFF1306\"");
assert_multiply("NaN", "0xFF1306");

trace("//\"-0.0\" * \"0xFF1306\"");
assert_multiply("-0.0", "0xFF1306");

trace("//\"Infinity\" * \"0xFF1306\"");
assert_multiply("Infinity", "0xFF1306");

trace("//\"1.0\" * \"0xFF1306\"");
assert_multiply("1.0", "0xFF1306");

trace("//\"-1.0\" * \"0xFF1306\"");
assert_multiply("-1.0", "0xFF1306");

trace("//\"0xFF1306\" * \"0xFF1306\"");
assert_multiply("0xFF1306", "0xFF1306");