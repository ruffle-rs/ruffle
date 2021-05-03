package {
	public class Test {
	}
}

function assert_add(val1, val2) {
	trace(val1 + val2);
}

trace("//true + true");
assert_add(true, true);

trace("//false + true");
assert_add(false, true);

trace("//null + true");
assert_add(null, true);

trace("//undefined + true");
assert_add(undefined, true);

trace("//\"\" + true");
assert_add("", true);

trace("//\"str\" + true");
assert_add("str", true);

trace("//\"true\" + true");
assert_add("true", true);

trace("//\"false\" + true");
assert_add("false", true);

trace("//0.0 + true");
assert_add(0.0, true);

trace("//NaN + true");
assert_add(NaN, true);

trace("//-0.0 + true");
assert_add(-0.0, true);

trace("//Infinity + true");
assert_add(Infinity, true);

trace("//1.0 + true");
assert_add(1.0, true);

trace("//-1.0 + true");
assert_add(-1.0, true);

trace("//0xFF1306 + true");
assert_add(0xFF1306, true);

trace("//new Object() + true");
assert_add({}, true);

trace("//\"0.0\" + true");
assert_add("0.0", true);

trace("//\"NaN\" + true");
assert_add("NaN", true);

trace("//\"-0.0\" + true");
assert_add("-0.0", true);

trace("//\"Infinity\" + true");
assert_add("Infinity", true);

trace("//\"1.0\" + true");
assert_add("1.0", true);

trace("//\"-1.0\" + true");
assert_add("-1.0", true);

trace("//\"0xFF1306\" + true");
assert_add("0xFF1306", true);

trace("//true + false");
assert_add(true, false);

trace("//false + false");
assert_add(false, false);

trace("//null + false");
assert_add(null, false);

trace("//undefined + false");
assert_add(undefined, false);

trace("//\"\" + false");
assert_add("", false);

trace("//\"str\" + false");
assert_add("str", false);

trace("//\"true\" + false");
assert_add("true", false);

trace("//\"false\" + false");
assert_add("false", false);

trace("//0.0 + false");
assert_add(0.0, false);

trace("//NaN + false");
assert_add(NaN, false);

trace("//-0.0 + false");
assert_add(-0.0, false);

trace("//Infinity + false");
assert_add(Infinity, false);

trace("//1.0 + false");
assert_add(1.0, false);

trace("//-1.0 + false");
assert_add(-1.0, false);

trace("//0xFF1306 + false");
assert_add(0xFF1306, false);

trace("//new Object() + false");
assert_add({}, false);

trace("//\"0.0\" + false");
assert_add("0.0", false);

trace("//\"NaN\" + false");
assert_add("NaN", false);

trace("//\"-0.0\" + false");
assert_add("-0.0", false);

trace("//\"Infinity\" + false");
assert_add("Infinity", false);

trace("//\"1.0\" + false");
assert_add("1.0", false);

trace("//\"-1.0\" + false");
assert_add("-1.0", false);

trace("//\"0xFF1306\" + false");
assert_add("0xFF1306", false);
trace("//true + null");
assert_add(true, null);

trace("//false + null");
assert_add(false, null);

trace("//null + null");
assert_add(null, null);

trace("//undefined + null");
assert_add(undefined, null);

trace("//\"\" + null");
assert_add("", null);

trace("//\"str\" + null");
assert_add("str", null);

trace("//\"true\" + null");
assert_add("true", null);

trace("//\"false\" + null");
assert_add("false", null);

trace("//0.0 + null");
assert_add(0.0, null);

trace("//NaN + null");
assert_add(NaN, null);

trace("//-0.0 + null");
assert_add(-0.0, null);

trace("//Infinity + null");
assert_add(Infinity, null);

trace("//1.0 + null");
assert_add(1.0, null);

trace("//-1.0 + null");
assert_add(-1.0, null);

trace("//0xFF1306 + null");
assert_add(0xFF1306, null);

trace("//new Object() + null");
assert_add({}, null);

trace("//\"0.0\" + null");
assert_add("0.0", null);

trace("//\"NaN\" + null");
assert_add("NaN", null);

trace("//\"-0.0\" + null");
assert_add("-0.0", null);

trace("//\"Infinity\" + null");
assert_add("Infinity", null);

trace("//\"1.0\" + null");
assert_add("1.0", null);

trace("//\"-1.0\" + null");
assert_add("-1.0", null);

trace("//\"0xFF1306\" + null");
assert_add("0xFF1306", null);

trace("//true + undefined");
assert_add(true, undefined);

trace("//false + undefined");
assert_add(false, undefined);

trace("//null + undefined");
assert_add(null, undefined);

trace("//undefined + undefined");
assert_add(undefined, undefined);

trace("//\"\" + undefined");
assert_add("", undefined);

trace("//\"str\" + undefined");
assert_add("str", undefined);

trace("//\"true\" + undefined");
assert_add("true", undefined);

trace("//\"false\" + undefined");
assert_add("false", undefined);

trace("//0.0 + undefined");
assert_add(0.0, undefined);

trace("//NaN + undefined");
assert_add(NaN, undefined);

trace("//-0.0 + undefined");
assert_add(-0.0, undefined);

trace("//Infinity + undefined");
assert_add(Infinity, undefined);

trace("//1.0 + undefined");
assert_add(1.0, undefined);

trace("//-1.0 + undefined");
assert_add(-1.0, undefined);

trace("//0xFF1306 + undefined");
assert_add(0xFF1306, undefined);

trace("//new Object() + undefined");
assert_add({}, undefined);

trace("//\"0.0\" + undefined");
assert_add("0.0", undefined);

trace("//\"NaN\" + undefined");
assert_add("NaN", undefined);

trace("//\"-0.0\" + undefined");
assert_add("-0.0", undefined);

trace("//\"Infinity\" + undefined");
assert_add("Infinity", undefined);

trace("//\"1.0\" + undefined");
assert_add("1.0", undefined);

trace("//\"-1.0\" + undefined");
assert_add("-1.0", undefined);

trace("//\"0xFF1306\" + undefined");
assert_add("0xFF1306", undefined);

trace("//true + \"\"");
assert_add(true, "");

trace("//false + \"\"");
assert_add(false, "");

trace("//null + \"\"");
assert_add(null, "");

trace("//undefined + \"\"");
assert_add(undefined, "");

trace("//\"\" + \"\"");
assert_add("", "");

trace("//\"str\" + \"\"");
assert_add("str", "");

trace("//\"true\" + \"\"");
assert_add("true", "");

trace("//\"false\" + \"\"");
assert_add("false", "");

trace("//0.0 + \"\"");
assert_add(0.0, "");

trace("//NaN + \"\"");
assert_add(NaN, "");

trace("//-0.0 + \"\"");
assert_add(-0.0, "");

trace("//Infinity + \"\"");
assert_add(Infinity, "");

trace("//1.0 + \"\"");
assert_add(1.0, "");

trace("//-1.0 + \"\"");
assert_add(-1.0, "");

trace("//0xFF1306 + \"\"");
assert_add(0xFF1306, "");

trace("//new Object() + \"\"");
assert_add({}, "");

trace("//\"0.0\" + \"\"");
assert_add("0.0", "");

trace("//\"NaN\" + \"\"");
assert_add("NaN", "");

trace("//\"-0.0\" + \"\"");
assert_add("-0.0", "");

trace("//\"Infinity\" + \"\"");
assert_add("Infinity", "");

trace("//\"1.0\" + \"\"");
assert_add("1.0", "");

trace("//\"-1.0\" + \"\"");
assert_add("-1.0", "");

trace("//\"0xFF1306\" + \"\"");
assert_add("0xFF1306", "");

trace("//true + \"str\"");
assert_add(true, "str");

trace("//false + \"str\"");
assert_add(false, "str");

trace("//null + \"str\"");
assert_add(null, "str");

trace("//undefined + \"str\"");
assert_add(undefined, "str");

trace("//\"\" + \"str\"");
assert_add("", "str");

trace("//\"str\" + \"str\"");
assert_add("str", "str");

trace("//\"true\" + \"str\"");
assert_add("true", "str");

trace("//\"false\" + \"str\"");
assert_add("false", "str");

trace("//0.0 + \"str\"");
assert_add(0.0, "str");

trace("//NaN + \"str\"");
assert_add(NaN, "str");

trace("//-0.0 + \"str\"");
assert_add(-0.0, "str");

trace("//Infinity + \"str\"");
assert_add(Infinity, "str");

trace("//1.0 + \"str\"");
assert_add(1.0, "str");

trace("//-1.0 + \"str\"");
assert_add(-1.0, "str");

trace("//0xFF1306 + \"str\"");
assert_add(0xFF1306, "str");

trace("//new Object() + \"str\"");
assert_add({}, "str");

trace("//\"0.0\" + \"str\"");
assert_add("0.0", "str");

trace("//\"NaN\" + \"str\"");
assert_add("NaN", "str");

trace("//\"-0.0\" + \"str\"");
assert_add("-0.0", "str");

trace("//\"Infinity\" + \"str\"");
assert_add("Infinity", "str");

trace("//\"1.0\" + \"str\"");
assert_add("1.0", "str");

trace("//\"-1.0\" + \"str\"");
assert_add("-1.0", "str");

trace("//\"0xFF1306\" + \"str\"");
assert_add("0xFF1306", "str");

trace("//true + \"true\"");
assert_add(true, "true");

trace("//false + \"true\"");
assert_add(false, "true");

trace("//null + \"true\"");
assert_add(null, "true");

trace("//undefined + \"true\"");
assert_add(undefined, "true");

trace("//\"\" + \"true\"");
assert_add("", "true");

trace("//\"str\" + \"true\"");
assert_add("str", "true");

trace("//\"true\" + \"true\"");
assert_add("true", "true");

trace("//\"false\" + \"true\"");
assert_add("false", "true");

trace("//0.0 + \"true\"");
assert_add(0.0, "true");

trace("//NaN + \"true\"");
assert_add(NaN, "true");

trace("//-0.0 + \"true\"");
assert_add(-0.0, "true");

trace("//Infinity + \"true\"");
assert_add(Infinity, "true");

trace("//1.0 + \"true\"");
assert_add(1.0, "true");

trace("//-1.0 + \"true\"");
assert_add(-1.0, "true");

trace("//0xFF1306 + \"true\"");
assert_add(0xFF1306, "true");

trace("//new Object() + \"true\"");
assert_add({}, "true");

trace("//\"0.0\" + \"true\"");
assert_add("0.0", "true");

trace("//\"NaN\" + \"true\"");
assert_add("NaN", "true");

trace("//\"-0.0\" + \"true\"");
assert_add("-0.0", "true");

trace("//\"Infinity\" + \"true\"");
assert_add("Infinity", "true");

trace("//\"1.0\" + \"true\"");
assert_add("1.0", "true");

trace("//\"-1.0\" + \"true\"");
assert_add("-1.0", "true");

trace("//\"0xFF1306\" + \"true\"");
assert_add("0xFF1306", "true");

trace("//true + \"false\"");
assert_add(true, "false");

trace("//false + \"false\"");
assert_add(false, "false");

trace("//null + \"false\"");
assert_add(null, "false");

trace("//undefined + \"false\"");
assert_add(undefined, "false");

trace("//\"\" + \"false\"");
assert_add("", "false");

trace("//\"str\" + \"false\"");
assert_add("str", "false");

trace("//\"true\" + \"false\"");
assert_add("true", "false");

trace("//\"false\" + \"false\"");
assert_add("false", "false");

trace("//0.0 + \"false\"");
assert_add(0.0, "false");

trace("//NaN + \"false\"");
assert_add(NaN, "false");

trace("//-0.0 + \"false\"");
assert_add(-0.0, "false");

trace("//Infinity + \"false\"");
assert_add(Infinity, "false");

trace("//1.0 + \"false\"");
assert_add(1.0, "false");

trace("//-1.0 + \"false\"");
assert_add(-1.0, "false");

trace("//0xFF1306 + \"false\"");
assert_add(0xFF1306, "false");

trace("//new Object() + \"false\"");
assert_add({}, "false");

trace("//\"0.0\" + \"false\"");
assert_add("0.0", "false");

trace("//\"NaN\" + \"false\"");
assert_add("NaN", "false");

trace("//\"-0.0\" + \"false\"");
assert_add("-0.0", "false");

trace("//\"Infinity\" + \"false\"");
assert_add("Infinity", "false");

trace("//\"1.0\" + \"false\"");
assert_add("1.0", "false");

trace("//\"-1.0\" + \"false\"");
assert_add("-1.0", "false");

trace("//\"0xFF1306\" + \"false\"");
assert_add("0xFF1306", "false");

trace("//true + 0.0");
assert_add(true, 0.0);

trace("//false + 0.0");
assert_add(false, 0.0);

trace("//null + 0.0");
assert_add(null, 0.0);

trace("//undefined + 0.0");
assert_add(undefined, 0.0);

trace("//\"\" + 0.0");
assert_add("", 0.0);

trace("//\"str\" + 0.0");
assert_add("str", 0.0);

trace("//\"true\" + 0.0");
assert_add("true", 0.0);

trace("//\"false\" + 0.0");
assert_add("false", 0.0);

trace("//0.0 + 0.0");
assert_add(0.0, 0.0);

trace("//NaN + 0.0");
assert_add(NaN, 0.0);

trace("//-0.0 + 0.0");
assert_add(-0.0, 0.0);

trace("//Infinity + 0.0");
assert_add(Infinity, 0.0);

trace("//1.0 + 0.0");
assert_add(1.0, 0.0);

trace("//-1.0 + 0.0");
assert_add(-1.0, 0.0);

trace("//0xFF1306 + 0.0");
assert_add(0xFF1306, 0.0);

trace("//new Object() + 0.0");
assert_add({}, 0.0);

trace("//\"0.0\" + 0.0");
assert_add("0.0", 0.0);

trace("//\"NaN\" + 0.0");
assert_add("NaN", 0.0);

trace("//\"-0.0\" + 0.0");
assert_add("-0.0", 0.0);

trace("//\"Infinity\" + 0.0");
assert_add("Infinity", 0.0);

trace("//\"1.0\" + 0.0");
assert_add("1.0", 0.0);

trace("//\"-1.0\" + 0.0");
assert_add("-1.0", 0.0);

trace("//\"0xFF1306\" + 0.0");
assert_add("0xFF1306", 0.0);

trace("//true + NaN");
assert_add(true, NaN);

trace("//false + NaN");
assert_add(false, NaN);

trace("//null + NaN");
assert_add(null, NaN);

trace("//undefined + NaN");
assert_add(undefined, NaN);

trace("//\"\" + NaN");
assert_add("", NaN);

trace("//\"str\" + NaN");
assert_add("str", NaN);

trace("//\"true\" + NaN");
assert_add("true", NaN);

trace("//\"false\" + NaN");
assert_add("false", NaN);

trace("//0.0 + NaN");
assert_add(0.0, NaN);

trace("//NaN + NaN");
assert_add(NaN, NaN);

trace("//-0.0 + NaN");
assert_add(-0.0, NaN);

trace("//Infinity + NaN");
assert_add(Infinity, NaN);

trace("//1.0 + NaN");
assert_add(1.0, NaN);

trace("//-1.0 + NaN");
assert_add(-1.0, NaN);

trace("//0xFF1306 + NaN");
assert_add(0xFF1306, NaN);

trace("//new Object() + NaN");
assert_add({}, NaN);

trace("//\"0.0\" + NaN");
assert_add("0.0", NaN);

trace("//\"NaN\" + NaN");
assert_add("NaN", NaN);

trace("//\"-0.0\" + NaN");
assert_add("-0.0", NaN);

trace("//\"Infinity\" + NaN");
assert_add("Infinity", NaN);

trace("//\"1.0\" + NaN");
assert_add("1.0", NaN);

trace("//\"-1.0\" + NaN");
assert_add("-1.0", NaN);

trace("//\"0xFF1306\" + NaN");
assert_add("0xFF1306", NaN);

trace("//true + -0.0");
assert_add(true, -0.0);

trace("//false + -0.0");
assert_add(false, -0.0);

trace("//null + -0.0");
assert_add(null, -0.0);

trace("//undefined + -0.0");
assert_add(undefined, -0.0);

trace("//\"\" + -0.0");
assert_add("", -0.0);

trace("//\"str\" + -0.0");
assert_add("str", -0.0);

trace("//\"true\" + -0.0");
assert_add("true", -0.0);

trace("//\"false\" + -0.0");
assert_add("false", -0.0);

trace("//0.0 + -0.0");
assert_add(0.0, -0.0);

trace("//NaN + -0.0");
assert_add(NaN, -0.0);

trace("//-0.0 + -0.0");
assert_add(-0.0, -0.0);

trace("//Infinity + -0.0");
assert_add(Infinity, -0.0);

trace("//1.0 + -0.0");
assert_add(1.0, -0.0);

trace("//-1.0 + -0.0");
assert_add(-1.0, -0.0);

trace("//0xFF1306 + -0.0");
assert_add(0xFF1306, -0.0);

trace("//new Object() + -0.0");
assert_add({}, -0.0);

trace("//\"0.0\" + -0.0");
assert_add("0.0", -0.0);

trace("//\"NaN\" + -0.0");
assert_add("NaN", -0.0);

trace("//\"-0.0\" + -0.0");
assert_add("-0.0", -0.0);

trace("//\"Infinity\" + -0.0");
assert_add("Infinity", -0.0);

trace("//\"1.0\" + -0.0");
assert_add("1.0", -0.0);

trace("//\"-1.0\" + -0.0");
assert_add("-1.0", -0.0);

trace("//\"0xFF1306\" + -0.0");
assert_add("0xFF1306", -0.0);

trace("//true + Infinity");
assert_add(true, Infinity);

trace("//false + Infinity");
assert_add(false, Infinity);

trace("//null + Infinity");
assert_add(null, Infinity);

trace("//undefined + Infinity");
assert_add(undefined, Infinity);

trace("//\"\" + Infinity");
assert_add("", Infinity);

trace("//\"str\" + Infinity");
assert_add("str", Infinity);

trace("//\"true\" + Infinity");
assert_add("true", Infinity);

trace("//\"false\" + Infinity");
assert_add("false", Infinity);

trace("//0.0 + Infinity");
assert_add(0.0, Infinity);

trace("//NaN + Infinity");
assert_add(NaN, Infinity);

trace("//-0.0 + Infinity");
assert_add(-0.0, Infinity);

trace("//Infinity + Infinity");
assert_add(Infinity, Infinity);

trace("//1.0 + Infinity");
assert_add(1.0, Infinity);

trace("//-1.0 + Infinity");
assert_add(-1.0, Infinity);

trace("//0xFF1306 + Infinity");
assert_add(0xFF1306, Infinity);

trace("//new Object() + Infinity");
assert_add({}, Infinity);

trace("//\"0.0\" + Infinity");
assert_add("0.0", Infinity);

trace("//\"NaN\" + Infinity");
assert_add("NaN", Infinity);

trace("//\"-0.0\" + Infinity");
assert_add("-0.0", Infinity);

trace("//\"Infinity\" + Infinity");
assert_add("Infinity", Infinity);

trace("//\"1.0\" + Infinity");
assert_add("1.0", Infinity);

trace("//\"-1.0\" + Infinity");
assert_add("-1.0", Infinity);

trace("//\"0xFF1306\" + Infinity");
assert_add("0xFF1306", Infinity);

trace("//true + 1.0");
assert_add(true, 1.0);

trace("//false + 1.0");
assert_add(false, 1.0);

trace("//null + 1.0");
assert_add(null, 1.0);

trace("//undefined + 1.0");
assert_add(undefined, 1.0);

trace("//\"\" + 1.0");
assert_add("", 1.0);

trace("//\"str\" + 1.0");
assert_add("str", 1.0);

trace("//\"true\" + 1.0");
assert_add("true", 1.0);

trace("//\"false\" + 1.0");
assert_add("false", 1.0);

trace("//0.0 + 1.0");
assert_add(0.0, 1.0);

trace("//NaN + 1.0");
assert_add(NaN, 1.0);

trace("//-0.0 + 1.0");
assert_add(-0.0, 1.0);

trace("//Infinity + 1.0");
assert_add(Infinity, 1.0);

trace("//1.0 + 1.0");
assert_add(1.0, 1.0);

trace("//-1.0 + 1.0");
assert_add(-1.0, 1.0);

trace("//0xFF1306 + 1.0");
assert_add(0xFF1306, 1.0);

trace("//new Object() + 1.0");
assert_add({}, 1.0);

trace("//\"0.0\" + 1.0");
assert_add("0.0", 1.0);

trace("//\"NaN\" + 1.0");
assert_add("NaN", 1.0);

trace("//\"-0.0\" + 1.0");
assert_add("-0.0", 1.0);

trace("//\"Infinity\" + 1.0");
assert_add("Infinity", 1.0);

trace("//\"1.0\" + 1.0");
assert_add("1.0", 1.0);

trace("//\"-1.0\" + 1.0");
assert_add("-1.0", 1.0);

trace("//\"0xFF1306\" + 1.0");
assert_add("0xFF1306", 1.0);

trace("//true + -1.0");
assert_add(true, -1.0);

trace("//false + -1.0");
assert_add(false, -1.0);

trace("//null + -1.0");
assert_add(null, -1.0);

trace("//undefined + -1.0");
assert_add(undefined, -1.0);

trace("//\"\" + -1.0");
assert_add("", -1.0);

trace("//\"str\" + -1.0");
assert_add("str", -1.0);

trace("//\"true\" + -1.0");
assert_add("true", -1.0);

trace("//\"false\" + -1.0");
assert_add("false", -1.0);

trace("//0.0 + -1.0");
assert_add(0.0, -1.0);

trace("//NaN + -1.0");
assert_add(NaN, -1.0);

trace("//-0.0 + -1.0");
assert_add(-0.0, -1.0);

trace("//Infinity + -1.0");
assert_add(Infinity, -1.0);

trace("//1.0 + -1.0");
assert_add(1.0, -1.0);

trace("//-1.0 + -1.0");
assert_add(-1.0, -1.0);

trace("//0xFF1306 + -1.0");
assert_add(0xFF1306, -1.0);

trace("//new Object() + -1.0");
assert_add({}, -1.0);

trace("//\"0.0\" + -1.0");
assert_add("0.0", -1.0);

trace("//\"NaN\" + -1.0");
assert_add("NaN", -1.0);

trace("//\"-0.0\" + -1.0");
assert_add("-0.0", -1.0);

trace("//\"Infinity\" + -1.0");
assert_add("Infinity", -1.0);

trace("//\"1.0\" + -1.0");
assert_add("1.0", -1.0);

trace("//\"-1.0\" + -1.0");
assert_add("-1.0", -1.0);

trace("//\"0xFF1306\" + -1.0");
assert_add("0xFF1306", -1.0);

trace("//true + 0xFF1306");
assert_add(true, 0xFF1306);

trace("//false + 0xFF1306");
assert_add(false, 0xFF1306);

trace("//null + 0xFF1306");
assert_add(null, 0xFF1306);

trace("//undefined + 0xFF1306");
assert_add(undefined, 0xFF1306);

trace("//\"\" + 0xFF1306");
assert_add("", 0xFF1306);

trace("//\"str\" + 0xFF1306");
assert_add("str", 0xFF1306);

trace("//\"true\" + 0xFF1306");
assert_add("true", 0xFF1306);

trace("//\"false\" + 0xFF1306");
assert_add("false", 0xFF1306);

trace("//0.0 + 0xFF1306");
assert_add(0.0, 0xFF1306);

trace("//NaN + 0xFF1306");
assert_add(NaN, 0xFF1306);

trace("//-0.0 + 0xFF1306");
assert_add(-0.0, 0xFF1306);

trace("//Infinity + 0xFF1306");
assert_add(Infinity, 0xFF1306);

trace("//1.0 + 0xFF1306");
assert_add(1.0, 0xFF1306);

trace("//-1.0 + 0xFF1306");
assert_add(-1.0, 0xFF1306);

trace("//0xFF1306 + 0xFF1306");
assert_add(0xFF1306, 0xFF1306);

trace("//new Object() + 0xFF1306");
assert_add({}, 0xFF1306);

trace("//\"0.0\" + 0xFF1306");
assert_add("0.0", 0xFF1306);

trace("//\"NaN\" + 0xFF1306");
assert_add("NaN", 0xFF1306);

trace("//\"-0.0\" + 0xFF1306");
assert_add("-0.0", 0xFF1306);

trace("//\"Infinity\" + 0xFF1306");
assert_add("Infinity", 0xFF1306);

trace("//\"1.0\" + 0xFF1306");
assert_add("1.0", 0xFF1306);

trace("//\"-1.0\" + 0xFF1306");
assert_add("-1.0", 0xFF1306);

trace("//\"0xFF1306\" + 0xFF1306");
assert_add("0xFF1306", 0xFF1306);

trace("//true + new Object()");
assert_add(true, {});

trace("//false + new Object()");
assert_add(false, {});

trace("//null + new Object()");
assert_add(null, {});

trace("//undefined + new Object()");
assert_add(undefined, {});

trace("//\"\" + new Object()");
assert_add("", {});

trace("//\"str\" + new Object()");
assert_add("str", {});

trace("//\"true\" + new Object()");
assert_add("true", {});

trace("//\"false\" + new Object()");
assert_add("false", {});

trace("//0.0 + new Object()");
assert_add(0.0, {});

trace("//NaN + new Object()");
assert_add(NaN, {});

trace("//-0.0 + new Object()");
assert_add(-0.0, {});

trace("//Infinity + new Object()");
assert_add(Infinity, {});

trace("//1.0 + new Object()");
assert_add(1.0, {});

trace("//-1.0 + new Object()");
assert_add(-1.0, {});

trace("//0xFF1306 + new Object()");
assert_add(0xFF1306, {});

trace("//new Object() + new Object()");
assert_add({}, {});

trace("//\"0.0\" + new Object()");
assert_add("0.0", {});

trace("//\"NaN\" + new Object()");
assert_add("NaN", {});

trace("//\"-0.0\" + new Object()");
assert_add("-0.0", {});

trace("//\"Infinity\" + new Object()");
assert_add("Infinity", {});

trace("//\"1.0\" + new Object()");
assert_add("1.0", {});

trace("//\"-1.0\" + new Object()");
assert_add("-1.0", {});

trace("//\"0xFF1306\" + new Object()");
assert_add("0xFF1306", {});

trace("//true + \"0.0\"");
assert_add(true, "0.0");

trace("//false + \"0.0\"");
assert_add(false, "0.0");

trace("//null + \"0.0\"");
assert_add(null, "0.0");

trace("//undefined + \"0.0\"");
assert_add(undefined, "0.0");

trace("//\"\" + \"0.0\"");
assert_add("", "0.0");

trace("//\"str\" + \"0.0\"");
assert_add("str", "0.0");

trace("//\"true\" + \"0.0\"");
assert_add("true", "0.0");

trace("//\"false\" + \"0.0\"");
assert_add("false", "0.0");

trace("//0.0 + \"0.0\"");
assert_add(0.0, "0.0");

trace("//NaN + \"0.0\"");
assert_add(NaN, "0.0");

trace("//-0.0 + \"0.0\"");
assert_add(-0.0, "0.0");

trace("//Infinity + \"0.0\"");
assert_add(Infinity, "0.0");

trace("//1.0 + \"0.0\"");
assert_add(1.0, "0.0");

trace("//-1.0 + \"0.0\"");
assert_add(-1.0, "0.0");

trace("//0xFF1306 + \"0.0\"");
assert_add(0xFF1306, "0.0");

trace("//new Object() + \"0.0\"");
assert_add({}, "0.0");

trace("//\"0.0\" + \"0.0\"");
assert_add("0.0", "0.0");

trace("//\"NaN\" + \"0.0\"");
assert_add("NaN", "0.0");

trace("//\"-0.0\" + \"0.0\"");
assert_add("-0.0", "0.0");

trace("//\"Infinity\" + \"0.0\"");
assert_add("Infinity", "0.0");

trace("//\"1.0\" + \"0.0\"");
assert_add("1.0", "0.0");

trace("//\"-1.0\" + \"0.0\"");
assert_add("-1.0", "0.0");

trace("//\"0xFF1306\" + \"0.0\"");
assert_add("0xFF1306", "0.0");

trace("//true + \"NaN\"");
assert_add(true, "NaN");

trace("//false + \"NaN\"");
assert_add(false, "NaN");

trace("//null + \"NaN\"");
assert_add(null, "NaN");

trace("//undefined + \"NaN\"");
assert_add(undefined, "NaN");

trace("//\"\" + \"NaN\"");
assert_add("", "NaN");

trace("//\"str\" + \"NaN\"");
assert_add("str", "NaN");

trace("//\"true\" + \"NaN\"");
assert_add("true", "NaN");

trace("//\"false\" + \"NaN\"");
assert_add("false", "NaN");

trace("//0.0 + \"NaN\"");
assert_add(0.0, "NaN");

trace("//NaN + \"NaN\"");
assert_add(NaN, "NaN");

trace("//-0.0 + \"NaN\"");
assert_add(-0.0, "NaN");

trace("//Infinity + \"NaN\"");
assert_add(Infinity, "NaN");

trace("//1.0 + \"NaN\"");
assert_add(1.0, "NaN");

trace("//-1.0 + \"NaN\"");
assert_add(-1.0, "NaN");

trace("//0xFF1306 + \"NaN\"");
assert_add(0xFF1306, "NaN");

trace("//new Object() + \"NaN\"");
assert_add({}, "NaN");

trace("//\"0.0\" + \"NaN\"");
assert_add("0.0", "NaN");

trace("//\"NaN\" + \"NaN\"");
assert_add("NaN", "NaN");

trace("//\"-0.0\" + \"NaN\"");
assert_add("-0.0", "NaN");

trace("//\"Infinity\" + \"NaN\"");
assert_add("Infinity", "NaN");

trace("//\"1.0\" + \"NaN\"");
assert_add("1.0", "NaN");

trace("//\"-1.0\" + \"NaN\"");
assert_add("-1.0", "NaN");

trace("//\"0xFF1306\" + \"NaN\"");
assert_add("0xFF1306", "NaN");

trace("//true + \"-0.0\"");
assert_add(true, "-0.0");

trace("//false + \"-0.0\"");
assert_add(false, "-0.0");

trace("//null + \"-0.0\"");
assert_add(null, "-0.0");

trace("//undefined + \"-0.0\"");
assert_add(undefined, "-0.0");

trace("//\"\" + \"-0.0\"");
assert_add("", "-0.0");

trace("//\"str\" + \"-0.0\"");
assert_add("str", "-0.0");

trace("//\"true\" + \"-0.0\"");
assert_add("true", "-0.0");

trace("//\"false\" + \"-0.0\"");
assert_add("false", "-0.0");

trace("//0.0 + \"-0.0\"");
assert_add(0.0, "-0.0");

trace("//NaN + \"-0.0\"");
assert_add(NaN, "-0.0");

trace("//-0.0 + \"-0.0\"");
assert_add(-0.0, "-0.0");

trace("//Infinity + \"-0.0\"");
assert_add(Infinity, "-0.0");

trace("//1.0 + \"-0.0\"");
assert_add(1.0, "-0.0");

trace("//-1.0 + \"-0.0\"");
assert_add(-1.0, "-0.0");

trace("//0xFF1306 + \"-0.0\"");
assert_add(0xFF1306, "-0.0");

trace("//new Object() + \"-0.0\"");
assert_add({}, "-0.0");

trace("//\"0.0\" + \"-0.0\"");
assert_add("0.0", "-0.0");

trace("//\"NaN\" + \"-0.0\"");
assert_add("NaN", "-0.0");

trace("//\"-0.0\" + \"-0.0\"");
assert_add("-0.0", "-0.0");

trace("//\"Infinity\" + \"-0.0\"");
assert_add("Infinity", "-0.0");

trace("//\"1.0\" + \"-0.0\"");
assert_add("1.0", "-0.0");

trace("//\"-1.0\" + \"-0.0\"");
assert_add("-1.0", "-0.0");

trace("//\"0xFF1306\" + \"-0.0\"");
assert_add("0xFF1306", "-0.0");

trace("//true + \"Infinity\"");
assert_add(true, "Infinity");

trace("//false + \"Infinity\"");
assert_add(false, "Infinity");

trace("//null + \"Infinity\"");
assert_add(null, "Infinity");

trace("//undefined + \"Infinity\"");
assert_add(undefined, "Infinity");

trace("//\"\" + \"Infinity\"");
assert_add("", "Infinity");

trace("//\"str\" + \"Infinity\"");
assert_add("str", "Infinity");

trace("//\"true\" + \"Infinity\"");
assert_add("true", "Infinity");

trace("//\"false\" + \"Infinity\"");
assert_add("false", "Infinity");

trace("//0.0 + \"Infinity\"");
assert_add(0.0, "Infinity");

trace("//NaN + \"Infinity\"");
assert_add(NaN, "Infinity");

trace("//-0.0 + \"Infinity\"");
assert_add(-0.0, "Infinity");

trace("//Infinity + \"Infinity\"");
assert_add(Infinity, "Infinity");

trace("//1.0 + \"Infinity\"");
assert_add(1.0, "Infinity");

trace("//-1.0 + \"Infinity\"");
assert_add(-1.0, "Infinity");

trace("//0xFF1306 + \"Infinity\"");
assert_add(0xFF1306, "Infinity");

trace("//new Object() + \"Infinity\"");
assert_add({}, "Infinity");

trace("//\"0.0\" + \"Infinity\"");
assert_add("0.0", "Infinity");

trace("//\"NaN\" + \"Infinity\"");
assert_add("NaN", "Infinity");

trace("//\"-0.0\" + \"Infinity\"");
assert_add("-0.0", "Infinity");

trace("//\"Infinity\" + \"Infinity\"");
assert_add("Infinity", "Infinity");

trace("//\"1.0\" + \"Infinity\"");
assert_add("1.0", "Infinity");

trace("//\"-1.0\" + \"Infinity\"");
assert_add("-1.0", "Infinity");

trace("//\"0xFF1306\" + \"Infinity\"");
assert_add("0xFF1306", "Infinity");

trace("//true + \"1.0\"");
assert_add(true, "1.0");

trace("//false + \"1.0\"");
assert_add(false, "1.0");

trace("//null + \"1.0\"");
assert_add(null, "1.0");

trace("//undefined + \"1.0\"");
assert_add(undefined, "1.0");

trace("//\"\" + \"1.0\"");
assert_add("", "1.0");

trace("//\"str\" + \"1.0\"");
assert_add("str", "1.0");

trace("//\"true\" + \"1.0\"");
assert_add("true", "1.0");

trace("//\"false\" + \"1.0\"");
assert_add("false", "1.0");

trace("//0.0 + \"1.0\"");
assert_add(0.0, "1.0");

trace("//NaN + \"1.0\"");
assert_add(NaN, "1.0");

trace("//-0.0 + \"1.0\"");
assert_add(-0.0, "1.0");

trace("//Infinity + \"1.0\"");
assert_add(Infinity, "1.0");

trace("//1.0 + \"1.0\"");
assert_add(1.0, "1.0");

trace("//-1.0 + \"1.0\"");
assert_add(-1.0, "1.0");

trace("//0xFF1306 + \"1.0\"");
assert_add(0xFF1306, "1.0");

trace("//new Object() + \"1.0\"");
assert_add({}, "1.0");

trace("//\"0.0\" + \"1.0\"");
assert_add("0.0", "1.0");

trace("//\"NaN\" + \"1.0\"");
assert_add("NaN", "1.0");

trace("//\"-0.0\" + \"1.0\"");
assert_add("-0.0", "1.0");

trace("//\"Infinity\" + \"1.0\"");
assert_add("Infinity", "1.0");

trace("//\"1.0\" + \"1.0\"");
assert_add("1.0", "1.0");

trace("//\"-1.0\" + \"1.0\"");
assert_add("-1.0", "1.0");

trace("//\"0xFF1306\" + \"1.0\"");
assert_add("0xFF1306", "1.0");

trace("//true + \"-1.0\"");
assert_add(true, "-1.0");

trace("//false + \"-1.0\"");
assert_add(false, "-1.0");

trace("//null + \"-1.0\"");
assert_add(null, "-1.0");

trace("//undefined + \"-1.0\"");
assert_add(undefined, "-1.0");

trace("//\"\" + \"-1.0\"");
assert_add("", "-1.0");

trace("//\"str\" + \"-1.0\"");
assert_add("str", "-1.0");

trace("//\"true\" + \"-1.0\"");
assert_add("true", "-1.0");

trace("//\"false\" + \"-1.0\"");
assert_add("false", "-1.0");

trace("//0.0 + \"-1.0\"");
assert_add(0.0, "-1.0");

trace("//NaN + \"-1.0\"");
assert_add(NaN, "-1.0");

trace("//-0.0 + \"-1.0\"");
assert_add(-0.0, "-1.0");

trace("//Infinity + \"-1.0\"");
assert_add(Infinity, "-1.0");

trace("//1.0 + \"-1.0\"");
assert_add(1.0, "-1.0");

trace("//-1.0 + \"-1.0\"");
assert_add(-1.0, "-1.0");

trace("//0xFF1306 + \"-1.0\"");
assert_add(0xFF1306, "-1.0");

trace("//new Object() + \"-1.0\"");
assert_add({}, "-1.0");

trace("//\"0.0\" + \"-1.0\"");
assert_add("0.0", "-1.0");

trace("//\"NaN\" + \"-1.0\"");
assert_add("NaN", "-1.0");

trace("//\"-0.0\" + \"-1.0\"");
assert_add("-0.0", "-1.0");

trace("//\"Infinity\" + \"-1.0\"");
assert_add("Infinity", "-1.0");

trace("//\"1.0\" + \"-1.0\"");
assert_add("1.0", "-1.0");

trace("//\"-1.0\" + \"-1.0\"");
assert_add("-1.0", "-1.0");

trace("//\"0xFF1306\" + \"-1.0\"");
assert_add("0xFF1306", "-1.0");

trace("//true + \"0xFF1306\"");
assert_add(true, "0xFF1306");

trace("//false + \"0xFF1306\"");
assert_add(false, "0xFF1306");

trace("//null + \"0xFF1306\"");
assert_add(null, "0xFF1306");

trace("//undefined + \"0xFF1306\"");
assert_add(undefined, "0xFF1306");

trace("//\"\" + \"0xFF1306\"");
assert_add("", "0xFF1306");

trace("//\"str\" + \"0xFF1306\"");
assert_add("str", "0xFF1306");

trace("//\"true\" + \"0xFF1306\"");
assert_add("true", "0xFF1306");

trace("//\"false\" + \"0xFF1306\"");
assert_add("false", "0xFF1306");

trace("//0.0 + \"0xFF1306\"");
assert_add(0.0, "0xFF1306");

trace("//NaN + \"0xFF1306\"");
assert_add(NaN, "0xFF1306");

trace("//-0.0 + \"0xFF1306\"");
assert_add(-0.0, "0xFF1306");

trace("//Infinity + \"0xFF1306\"");
assert_add(Infinity, "0xFF1306");

trace("//1.0 + \"0xFF1306\"");
assert_add(1.0, "0xFF1306");

trace("//-1.0 + \"0xFF1306\"");
assert_add(-1.0, "0xFF1306");

trace("//0xFF1306 + \"0xFF1306\"");
assert_add(0xFF1306, "0xFF1306");

trace("//new Object() + \"0xFF1306\"");
assert_add({}, "0xFF1306");

trace("//\"0.0\" + \"0xFF1306\"");
assert_add("0.0", "0xFF1306");

trace("//\"NaN\" + \"0xFF1306\"");
assert_add("NaN", "0xFF1306");

trace("//\"-0.0\" + \"0xFF1306\"");
assert_add("-0.0", "0xFF1306");

trace("//\"Infinity\" + \"0xFF1306\"");
assert_add("Infinity", "0xFF1306");

trace("//\"1.0\" + \"0xFF1306\"");
assert_add("1.0", "0xFF1306");

trace("//\"-1.0\" + \"0xFF1306\"");
assert_add("-1.0", "0xFF1306");

trace("//\"0xFF1306\" + \"0xFF1306\"");
assert_add("0xFF1306", "0xFF1306");