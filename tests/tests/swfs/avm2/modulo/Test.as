package {
	public class Test {
	}
}

function assert_modulo(val1, val2) {
	trace(val1 % val2);
}

trace("//true % true");
assert_modulo(true, true);

trace("//false % true");
assert_modulo(false, true);

trace("//null % true");
assert_modulo(null, true);

trace("//undefined % true");
assert_modulo(undefined, true);

trace("//\"\" % true");
assert_modulo("", true);

trace("//\"str\" % true");
assert_modulo("str", true);

trace("//\"true\" % true");
assert_modulo("true", true);

trace("//\"false\" % true");
assert_modulo("false", true);

trace("//0.0 % true");
assert_modulo(0.0, true);

trace("//NaN % true");
assert_modulo(NaN, true);

trace("//-0.0 % true");
assert_modulo(-0.0, true);

trace("//Infinity % true");
assert_modulo(Infinity, true);

trace("//1.0 % true");
assert_modulo(1.0, true);

trace("//-1.0 % true");
assert_modulo(-1.0, true);

trace("//0xFF1306 % true");
assert_modulo(0xFF1306, true);

trace("//new Object() % true");
assert_modulo({}, true);

trace("//\"0.0\" % true");
assert_modulo("0.0", true);

trace("//\"NaN\" % true");
assert_modulo("NaN", true);

trace("//\"-0.0\" % true");
assert_modulo("-0.0", true);

trace("//\"Infinity\" % true");
assert_modulo("Infinity", true);

trace("//\"1.0\" % true");
assert_modulo("1.0", true);

trace("//\"-1.0\" % true");
assert_modulo("-1.0", true);

trace("//\"0xFF1306\" % true");
assert_modulo("0xFF1306", true);

trace("//true % false");
assert_modulo(true, false);

trace("//false % false");
assert_modulo(false, false);

trace("//null % false");
assert_modulo(null, false);

trace("//undefined % false");
assert_modulo(undefined, false);

trace("//\"\" % false");
assert_modulo("", false);

trace("//\"str\" % false");
assert_modulo("str", false);

trace("//\"true\" % false");
assert_modulo("true", false);

trace("//\"false\" % false");
assert_modulo("false", false);

trace("//0.0 % false");
assert_modulo(0.0, false);

trace("//NaN % false");
assert_modulo(NaN, false);

trace("//-0.0 % false");
assert_modulo(-0.0, false);

trace("//Infinity % false");
assert_modulo(Infinity, false);

trace("//1.0 % false");
assert_modulo(1.0, false);

trace("//-1.0 % false");
assert_modulo(-1.0, false);

trace("//0xFF1306 % false");
assert_modulo(0xFF1306, false);

trace("//new Object() % false");
assert_modulo({}, false);

trace("//\"0.0\" % false");
assert_modulo("0.0", false);

trace("//\"NaN\" % false");
assert_modulo("NaN", false);

trace("//\"-0.0\" % false");
assert_modulo("-0.0", false);

trace("//\"Infinity\" % false");
assert_modulo("Infinity", false);

trace("//\"1.0\" % false");
assert_modulo("1.0", false);

trace("//\"-1.0\" % false");
assert_modulo("-1.0", false);

trace("//\"0xFF1306\" % false");
assert_modulo("0xFF1306", false);
trace("//true % null");
assert_modulo(true, null);

trace("//false % null");
assert_modulo(false, null);

trace("//null % null");
assert_modulo(null, null);

trace("//undefined % null");
assert_modulo(undefined, null);

trace("//\"\" % null");
assert_modulo("", null);

trace("//\"str\" % null");
assert_modulo("str", null);

trace("//\"true\" % null");
assert_modulo("true", null);

trace("//\"false\" % null");
assert_modulo("false", null);

trace("//0.0 % null");
assert_modulo(0.0, null);

trace("//NaN % null");
assert_modulo(NaN, null);

trace("//-0.0 % null");
assert_modulo(-0.0, null);

trace("//Infinity % null");
assert_modulo(Infinity, null);

trace("//1.0 % null");
assert_modulo(1.0, null);

trace("//-1.0 % null");
assert_modulo(-1.0, null);

trace("//0xFF1306 % null");
assert_modulo(0xFF1306, null);

trace("//new Object() % null");
assert_modulo({}, null);

trace("//\"0.0\" % null");
assert_modulo("0.0", null);

trace("//\"NaN\" % null");
assert_modulo("NaN", null);

trace("//\"-0.0\" % null");
assert_modulo("-0.0", null);

trace("//\"Infinity\" % null");
assert_modulo("Infinity", null);

trace("//\"1.0\" % null");
assert_modulo("1.0", null);

trace("//\"-1.0\" % null");
assert_modulo("-1.0", null);

trace("//\"0xFF1306\" % null");
assert_modulo("0xFF1306", null);

trace("//true % undefined");
assert_modulo(true, undefined);

trace("//false % undefined");
assert_modulo(false, undefined);

trace("//null % undefined");
assert_modulo(null, undefined);

trace("//undefined % undefined");
assert_modulo(undefined, undefined);

trace("//\"\" % undefined");
assert_modulo("", undefined);

trace("//\"str\" % undefined");
assert_modulo("str", undefined);

trace("//\"true\" % undefined");
assert_modulo("true", undefined);

trace("//\"false\" % undefined");
assert_modulo("false", undefined);

trace("//0.0 % undefined");
assert_modulo(0.0, undefined);

trace("//NaN % undefined");
assert_modulo(NaN, undefined);

trace("//-0.0 % undefined");
assert_modulo(-0.0, undefined);

trace("//Infinity % undefined");
assert_modulo(Infinity, undefined);

trace("//1.0 % undefined");
assert_modulo(1.0, undefined);

trace("//-1.0 % undefined");
assert_modulo(-1.0, undefined);

trace("//0xFF1306 % undefined");
assert_modulo(0xFF1306, undefined);

trace("//new Object() % undefined");
assert_modulo({}, undefined);

trace("//\"0.0\" % undefined");
assert_modulo("0.0", undefined);

trace("//\"NaN\" % undefined");
assert_modulo("NaN", undefined);

trace("//\"-0.0\" % undefined");
assert_modulo("-0.0", undefined);

trace("//\"Infinity\" % undefined");
assert_modulo("Infinity", undefined);

trace("//\"1.0\" % undefined");
assert_modulo("1.0", undefined);

trace("//\"-1.0\" % undefined");
assert_modulo("-1.0", undefined);

trace("//\"0xFF1306\" % undefined");
assert_modulo("0xFF1306", undefined);

trace("//true % \"\"");
assert_modulo(true, "");

trace("//false % \"\"");
assert_modulo(false, "");

trace("//null % \"\"");
assert_modulo(null, "");

trace("//undefined % \"\"");
assert_modulo(undefined, "");

trace("//\"\" % \"\"");
assert_modulo("", "");

trace("//\"str\" % \"\"");
assert_modulo("str", "");

trace("//\"true\" % \"\"");
assert_modulo("true", "");

trace("//\"false\" % \"\"");
assert_modulo("false", "");

trace("//0.0 % \"\"");
assert_modulo(0.0, "");

trace("//NaN % \"\"");
assert_modulo(NaN, "");

trace("//-0.0 % \"\"");
assert_modulo(-0.0, "");

trace("//Infinity % \"\"");
assert_modulo(Infinity, "");

trace("//1.0 % \"\"");
assert_modulo(1.0, "");

trace("//-1.0 % \"\"");
assert_modulo(-1.0, "");

trace("//0xFF1306 % \"\"");
assert_modulo(0xFF1306, "");

trace("//new Object() % \"\"");
assert_modulo({}, "");

trace("//\"0.0\" % \"\"");
assert_modulo("0.0", "");

trace("//\"NaN\" % \"\"");
assert_modulo("NaN", "");

trace("//\"-0.0\" % \"\"");
assert_modulo("-0.0", "");

trace("//\"Infinity\" % \"\"");
assert_modulo("Infinity", "");

trace("//\"1.0\" % \"\"");
assert_modulo("1.0", "");

trace("//\"-1.0\" % \"\"");
assert_modulo("-1.0", "");

trace("//\"0xFF1306\" % \"\"");
assert_modulo("0xFF1306", "");

trace("//true % \"str\"");
assert_modulo(true, "str");

trace("//false % \"str\"");
assert_modulo(false, "str");

trace("//null % \"str\"");
assert_modulo(null, "str");

trace("//undefined % \"str\"");
assert_modulo(undefined, "str");

trace("//\"\" % \"str\"");
assert_modulo("", "str");

trace("//\"str\" % \"str\"");
assert_modulo("str", "str");

trace("//\"true\" % \"str\"");
assert_modulo("true", "str");

trace("//\"false\" % \"str\"");
assert_modulo("false", "str");

trace("//0.0 % \"str\"");
assert_modulo(0.0, "str");

trace("//NaN % \"str\"");
assert_modulo(NaN, "str");

trace("//-0.0 % \"str\"");
assert_modulo(-0.0, "str");

trace("//Infinity % \"str\"");
assert_modulo(Infinity, "str");

trace("//1.0 % \"str\"");
assert_modulo(1.0, "str");

trace("//-1.0 % \"str\"");
assert_modulo(-1.0, "str");

trace("//0xFF1306 % \"str\"");
assert_modulo(0xFF1306, "str");

trace("//new Object() % \"str\"");
assert_modulo({}, "str");

trace("//\"0.0\" % \"str\"");
assert_modulo("0.0", "str");

trace("//\"NaN\" % \"str\"");
assert_modulo("NaN", "str");

trace("//\"-0.0\" % \"str\"");
assert_modulo("-0.0", "str");

trace("//\"Infinity\" % \"str\"");
assert_modulo("Infinity", "str");

trace("//\"1.0\" % \"str\"");
assert_modulo("1.0", "str");

trace("//\"-1.0\" % \"str\"");
assert_modulo("-1.0", "str");

trace("//\"0xFF1306\" % \"str\"");
assert_modulo("0xFF1306", "str");

trace("//true % \"true\"");
assert_modulo(true, "true");

trace("//false % \"true\"");
assert_modulo(false, "true");

trace("//null % \"true\"");
assert_modulo(null, "true");

trace("//undefined % \"true\"");
assert_modulo(undefined, "true");

trace("//\"\" % \"true\"");
assert_modulo("", "true");

trace("//\"str\" % \"true\"");
assert_modulo("str", "true");

trace("//\"true\" % \"true\"");
assert_modulo("true", "true");

trace("//\"false\" % \"true\"");
assert_modulo("false", "true");

trace("//0.0 % \"true\"");
assert_modulo(0.0, "true");

trace("//NaN % \"true\"");
assert_modulo(NaN, "true");

trace("//-0.0 % \"true\"");
assert_modulo(-0.0, "true");

trace("//Infinity % \"true\"");
assert_modulo(Infinity, "true");

trace("//1.0 % \"true\"");
assert_modulo(1.0, "true");

trace("//-1.0 % \"true\"");
assert_modulo(-1.0, "true");

trace("//0xFF1306 % \"true\"");
assert_modulo(0xFF1306, "true");

trace("//new Object() % \"true\"");
assert_modulo({}, "true");

trace("//\"0.0\" % \"true\"");
assert_modulo("0.0", "true");

trace("//\"NaN\" % \"true\"");
assert_modulo("NaN", "true");

trace("//\"-0.0\" % \"true\"");
assert_modulo("-0.0", "true");

trace("//\"Infinity\" % \"true\"");
assert_modulo("Infinity", "true");

trace("//\"1.0\" % \"true\"");
assert_modulo("1.0", "true");

trace("//\"-1.0\" % \"true\"");
assert_modulo("-1.0", "true");

trace("//\"0xFF1306\" % \"true\"");
assert_modulo("0xFF1306", "true");

trace("//true % \"false\"");
assert_modulo(true, "false");

trace("//false % \"false\"");
assert_modulo(false, "false");

trace("//null % \"false\"");
assert_modulo(null, "false");

trace("//undefined % \"false\"");
assert_modulo(undefined, "false");

trace("//\"\" % \"false\"");
assert_modulo("", "false");

trace("//\"str\" % \"false\"");
assert_modulo("str", "false");

trace("//\"true\" % \"false\"");
assert_modulo("true", "false");

trace("//\"false\" % \"false\"");
assert_modulo("false", "false");

trace("//0.0 % \"false\"");
assert_modulo(0.0, "false");

trace("//NaN % \"false\"");
assert_modulo(NaN, "false");

trace("//-0.0 % \"false\"");
assert_modulo(-0.0, "false");

trace("//Infinity % \"false\"");
assert_modulo(Infinity, "false");

trace("//1.0 % \"false\"");
assert_modulo(1.0, "false");

trace("//-1.0 % \"false\"");
assert_modulo(-1.0, "false");

trace("//0xFF1306 % \"false\"");
assert_modulo(0xFF1306, "false");

trace("//new Object() % \"false\"");
assert_modulo({}, "false");

trace("//\"0.0\" % \"false\"");
assert_modulo("0.0", "false");

trace("//\"NaN\" % \"false\"");
assert_modulo("NaN", "false");

trace("//\"-0.0\" % \"false\"");
assert_modulo("-0.0", "false");

trace("//\"Infinity\" % \"false\"");
assert_modulo("Infinity", "false");

trace("//\"1.0\" % \"false\"");
assert_modulo("1.0", "false");

trace("//\"-1.0\" % \"false\"");
assert_modulo("-1.0", "false");

trace("//\"0xFF1306\" % \"false\"");
assert_modulo("0xFF1306", "false");

trace("//true % 0.0");
assert_modulo(true, 0.0);

trace("//false % 0.0");
assert_modulo(false, 0.0);

trace("//null % 0.0");
assert_modulo(null, 0.0);

trace("//undefined % 0.0");
assert_modulo(undefined, 0.0);

trace("//\"\" % 0.0");
assert_modulo("", 0.0);

trace("//\"str\" % 0.0");
assert_modulo("str", 0.0);

trace("//\"true\" % 0.0");
assert_modulo("true", 0.0);

trace("//\"false\" % 0.0");
assert_modulo("false", 0.0);

trace("//0.0 % 0.0");
assert_modulo(0.0, 0.0);

trace("//NaN % 0.0");
assert_modulo(NaN, 0.0);

trace("//-0.0 % 0.0");
assert_modulo(-0.0, 0.0);

trace("//Infinity % 0.0");
assert_modulo(Infinity, 0.0);

trace("//1.0 % 0.0");
assert_modulo(1.0, 0.0);

trace("//-1.0 % 0.0");
assert_modulo(-1.0, 0.0);

trace("//0xFF1306 % 0.0");
assert_modulo(0xFF1306, 0.0);

trace("//new Object() % 0.0");
assert_modulo({}, 0.0);

trace("//\"0.0\" % 0.0");
assert_modulo("0.0", 0.0);

trace("//\"NaN\" % 0.0");
assert_modulo("NaN", 0.0);

trace("//\"-0.0\" % 0.0");
assert_modulo("-0.0", 0.0);

trace("//\"Infinity\" % 0.0");
assert_modulo("Infinity", 0.0);

trace("//\"1.0\" % 0.0");
assert_modulo("1.0", 0.0);

trace("//\"-1.0\" % 0.0");
assert_modulo("-1.0", 0.0);

trace("//\"0xFF1306\" % 0.0");
assert_modulo("0xFF1306", 0.0);

trace("//true % NaN");
assert_modulo(true, NaN);

trace("//false % NaN");
assert_modulo(false, NaN);

trace("//null % NaN");
assert_modulo(null, NaN);

trace("//undefined % NaN");
assert_modulo(undefined, NaN);

trace("//\"\" % NaN");
assert_modulo("", NaN);

trace("//\"str\" % NaN");
assert_modulo("str", NaN);

trace("//\"true\" % NaN");
assert_modulo("true", NaN);

trace("//\"false\" % NaN");
assert_modulo("false", NaN);

trace("//0.0 % NaN");
assert_modulo(0.0, NaN);

trace("//NaN % NaN");
assert_modulo(NaN, NaN);

trace("//-0.0 % NaN");
assert_modulo(-0.0, NaN);

trace("//Infinity % NaN");
assert_modulo(Infinity, NaN);

trace("//1.0 % NaN");
assert_modulo(1.0, NaN);

trace("//-1.0 % NaN");
assert_modulo(-1.0, NaN);

trace("//0xFF1306 % NaN");
assert_modulo(0xFF1306, NaN);

trace("//new Object() % NaN");
assert_modulo({}, NaN);

trace("//\"0.0\" % NaN");
assert_modulo("0.0", NaN);

trace("//\"NaN\" % NaN");
assert_modulo("NaN", NaN);

trace("//\"-0.0\" % NaN");
assert_modulo("-0.0", NaN);

trace("//\"Infinity\" % NaN");
assert_modulo("Infinity", NaN);

trace("//\"1.0\" % NaN");
assert_modulo("1.0", NaN);

trace("//\"-1.0\" % NaN");
assert_modulo("-1.0", NaN);

trace("//\"0xFF1306\" % NaN");
assert_modulo("0xFF1306", NaN);

trace("//true % -0.0");
assert_modulo(true, -0.0);

trace("//false % -0.0");
assert_modulo(false, -0.0);

trace("//null % -0.0");
assert_modulo(null, -0.0);

trace("//undefined % -0.0");
assert_modulo(undefined, -0.0);

trace("//\"\" % -0.0");
assert_modulo("", -0.0);

trace("//\"str\" % -0.0");
assert_modulo("str", -0.0);

trace("//\"true\" % -0.0");
assert_modulo("true", -0.0);

trace("//\"false\" % -0.0");
assert_modulo("false", -0.0);

trace("//0.0 % -0.0");
assert_modulo(0.0, -0.0);

trace("//NaN % -0.0");
assert_modulo(NaN, -0.0);

trace("//-0.0 % -0.0");
assert_modulo(-0.0, -0.0);

trace("//Infinity % -0.0");
assert_modulo(Infinity, -0.0);

trace("//1.0 % -0.0");
assert_modulo(1.0, -0.0);

trace("//-1.0 % -0.0");
assert_modulo(-1.0, -0.0);

trace("//0xFF1306 % -0.0");
assert_modulo(0xFF1306, -0.0);

trace("//new Object() % -0.0");
assert_modulo({}, -0.0);

trace("//\"0.0\" % -0.0");
assert_modulo("0.0", -0.0);

trace("//\"NaN\" % -0.0");
assert_modulo("NaN", -0.0);

trace("//\"-0.0\" % -0.0");
assert_modulo("-0.0", -0.0);

trace("//\"Infinity\" % -0.0");
assert_modulo("Infinity", -0.0);

trace("//\"1.0\" % -0.0");
assert_modulo("1.0", -0.0);

trace("//\"-1.0\" % -0.0");
assert_modulo("-1.0", -0.0);

trace("//\"0xFF1306\" % -0.0");
assert_modulo("0xFF1306", -0.0);

trace("//true % Infinity");
assert_modulo(true, Infinity);

trace("//false % Infinity");
assert_modulo(false, Infinity);

trace("//null % Infinity");
assert_modulo(null, Infinity);

trace("//undefined % Infinity");
assert_modulo(undefined, Infinity);

trace("//\"\" % Infinity");
assert_modulo("", Infinity);

trace("//\"str\" % Infinity");
assert_modulo("str", Infinity);

trace("//\"true\" % Infinity");
assert_modulo("true", Infinity);

trace("//\"false\" % Infinity");
assert_modulo("false", Infinity);

trace("//0.0 % Infinity");
assert_modulo(0.0, Infinity);

trace("//NaN % Infinity");
assert_modulo(NaN, Infinity);

trace("//-0.0 % Infinity");
assert_modulo(-0.0, Infinity);

trace("//Infinity % Infinity");
assert_modulo(Infinity, Infinity);

trace("//1.0 % Infinity");
assert_modulo(1.0, Infinity);

trace("//-1.0 % Infinity");
assert_modulo(-1.0, Infinity);

trace("//0xFF1306 % Infinity");
assert_modulo(0xFF1306, Infinity);

trace("//new Object() % Infinity");
assert_modulo({}, Infinity);

trace("//\"0.0\" % Infinity");
assert_modulo("0.0", Infinity);

trace("//\"NaN\" % Infinity");
assert_modulo("NaN", Infinity);

trace("//\"-0.0\" % Infinity");
assert_modulo("-0.0", Infinity);

trace("//\"Infinity\" % Infinity");
assert_modulo("Infinity", Infinity);

trace("//\"1.0\" % Infinity");
assert_modulo("1.0", Infinity);

trace("//\"-1.0\" % Infinity");
assert_modulo("-1.0", Infinity);

trace("//\"0xFF1306\" % Infinity");
assert_modulo("0xFF1306", Infinity);

trace("//true % 1.0");
assert_modulo(true, 1.0);

trace("//false % 1.0");
assert_modulo(false, 1.0);

trace("//null % 1.0");
assert_modulo(null, 1.0);

trace("//undefined % 1.0");
assert_modulo(undefined, 1.0);

trace("//\"\" % 1.0");
assert_modulo("", 1.0);

trace("//\"str\" % 1.0");
assert_modulo("str", 1.0);

trace("//\"true\" % 1.0");
assert_modulo("true", 1.0);

trace("//\"false\" % 1.0");
assert_modulo("false", 1.0);

trace("//0.0 % 1.0");
assert_modulo(0.0, 1.0);

trace("//NaN % 1.0");
assert_modulo(NaN, 1.0);

trace("//-0.0 % 1.0");
assert_modulo(-0.0, 1.0);

trace("//Infinity % 1.0");
assert_modulo(Infinity, 1.0);

trace("//1.0 % 1.0");
assert_modulo(1.0, 1.0);

trace("//-1.0 % 1.0");
assert_modulo(-1.0, 1.0);

trace("//0xFF1306 % 1.0");
assert_modulo(0xFF1306, 1.0);

trace("//new Object() % 1.0");
assert_modulo({}, 1.0);

trace("//\"0.0\" % 1.0");
assert_modulo("0.0", 1.0);

trace("//\"NaN\" % 1.0");
assert_modulo("NaN", 1.0);

trace("//\"-0.0\" % 1.0");
assert_modulo("-0.0", 1.0);

trace("//\"Infinity\" % 1.0");
assert_modulo("Infinity", 1.0);

trace("//\"1.0\" % 1.0");
assert_modulo("1.0", 1.0);

trace("//\"-1.0\" % 1.0");
assert_modulo("-1.0", 1.0);

trace("//\"0xFF1306\" % 1.0");
assert_modulo("0xFF1306", 1.0);

trace("//true % -1.0");
assert_modulo(true, -1.0);

trace("//false % -1.0");
assert_modulo(false, -1.0);

trace("//null % -1.0");
assert_modulo(null, -1.0);

trace("//undefined % -1.0");
assert_modulo(undefined, -1.0);

trace("//\"\" % -1.0");
assert_modulo("", -1.0);

trace("//\"str\" % -1.0");
assert_modulo("str", -1.0);

trace("//\"true\" % -1.0");
assert_modulo("true", -1.0);

trace("//\"false\" % -1.0");
assert_modulo("false", -1.0);

trace("//0.0 % -1.0");
assert_modulo(0.0, -1.0);

trace("//NaN % -1.0");
assert_modulo(NaN, -1.0);

trace("//-0.0 % -1.0");
assert_modulo(-0.0, -1.0);

trace("//Infinity % -1.0");
assert_modulo(Infinity, -1.0);

trace("//1.0 % -1.0");
assert_modulo(1.0, -1.0);

trace("//-1.0 % -1.0");
assert_modulo(-1.0, -1.0);

trace("//0xFF1306 % -1.0");
assert_modulo(0xFF1306, -1.0);

trace("//new Object() % -1.0");
assert_modulo({}, -1.0);

trace("//\"0.0\" % -1.0");
assert_modulo("0.0", -1.0);

trace("//\"NaN\" % -1.0");
assert_modulo("NaN", -1.0);

trace("//\"-0.0\" % -1.0");
assert_modulo("-0.0", -1.0);

trace("//\"Infinity\" % -1.0");
assert_modulo("Infinity", -1.0);

trace("//\"1.0\" % -1.0");
assert_modulo("1.0", -1.0);

trace("//\"-1.0\" % -1.0");
assert_modulo("-1.0", -1.0);

trace("//\"0xFF1306\" % -1.0");
assert_modulo("0xFF1306", -1.0);

trace("//true % 0xFF1306");
assert_modulo(true, 0xFF1306);

trace("//false % 0xFF1306");
assert_modulo(false, 0xFF1306);

trace("//null % 0xFF1306");
assert_modulo(null, 0xFF1306);

trace("//undefined % 0xFF1306");
assert_modulo(undefined, 0xFF1306);

trace("//\"\" % 0xFF1306");
assert_modulo("", 0xFF1306);

trace("//\"str\" % 0xFF1306");
assert_modulo("str", 0xFF1306);

trace("//\"true\" % 0xFF1306");
assert_modulo("true", 0xFF1306);

trace("//\"false\" % 0xFF1306");
assert_modulo("false", 0xFF1306);

trace("//0.0 % 0xFF1306");
assert_modulo(0.0, 0xFF1306);

trace("//NaN % 0xFF1306");
assert_modulo(NaN, 0xFF1306);

trace("//-0.0 % 0xFF1306");
assert_modulo(-0.0, 0xFF1306);

trace("//Infinity % 0xFF1306");
assert_modulo(Infinity, 0xFF1306);

trace("//1.0 % 0xFF1306");
assert_modulo(1.0, 0xFF1306);

trace("//-1.0 % 0xFF1306");
assert_modulo(-1.0, 0xFF1306);

trace("//0xFF1306 % 0xFF1306");
assert_modulo(0xFF1306, 0xFF1306);

trace("//new Object() % 0xFF1306");
assert_modulo({}, 0xFF1306);

trace("//\"0.0\" % 0xFF1306");
assert_modulo("0.0", 0xFF1306);

trace("//\"NaN\" % 0xFF1306");
assert_modulo("NaN", 0xFF1306);

trace("//\"-0.0\" % 0xFF1306");
assert_modulo("-0.0", 0xFF1306);

trace("//\"Infinity\" % 0xFF1306");
assert_modulo("Infinity", 0xFF1306);

trace("//\"1.0\" % 0xFF1306");
assert_modulo("1.0", 0xFF1306);

trace("//\"-1.0\" % 0xFF1306");
assert_modulo("-1.0", 0xFF1306);

trace("//\"0xFF1306\" % 0xFF1306");
assert_modulo("0xFF1306", 0xFF1306);

trace("//true % new Object()");
assert_modulo(true, {});

trace("//false % new Object()");
assert_modulo(false, {});

trace("//null % new Object()");
assert_modulo(null, {});

trace("//undefined % new Object()");
assert_modulo(undefined, {});

trace("//\"\" % new Object()");
assert_modulo("", {});

trace("//\"str\" % new Object()");
assert_modulo("str", {});

trace("//\"true\" % new Object()");
assert_modulo("true", {});

trace("//\"false\" % new Object()");
assert_modulo("false", {});

trace("//0.0 % new Object()");
assert_modulo(0.0, {});

trace("//NaN % new Object()");
assert_modulo(NaN, {});

trace("//-0.0 % new Object()");
assert_modulo(-0.0, {});

trace("//Infinity % new Object()");
assert_modulo(Infinity, {});

trace("//1.0 % new Object()");
assert_modulo(1.0, {});

trace("//-1.0 % new Object()");
assert_modulo(-1.0, {});

trace("//0xFF1306 % new Object()");
assert_modulo(0xFF1306, {});

trace("//new Object() % new Object()");
assert_modulo({}, {});

trace("//\"0.0\" % new Object()");
assert_modulo("0.0", {});

trace("//\"NaN\" % new Object()");
assert_modulo("NaN", {});

trace("//\"-0.0\" % new Object()");
assert_modulo("-0.0", {});

trace("//\"Infinity\" % new Object()");
assert_modulo("Infinity", {});

trace("//\"1.0\" % new Object()");
assert_modulo("1.0", {});

trace("//\"-1.0\" % new Object()");
assert_modulo("-1.0", {});

trace("//\"0xFF1306\" % new Object()");
assert_modulo("0xFF1306", {});

trace("//true % \"0.0\"");
assert_modulo(true, "0.0");

trace("//false % \"0.0\"");
assert_modulo(false, "0.0");

trace("//null % \"0.0\"");
assert_modulo(null, "0.0");

trace("//undefined % \"0.0\"");
assert_modulo(undefined, "0.0");

trace("//\"\" % \"0.0\"");
assert_modulo("", "0.0");

trace("//\"str\" % \"0.0\"");
assert_modulo("str", "0.0");

trace("//\"true\" % \"0.0\"");
assert_modulo("true", "0.0");

trace("//\"false\" % \"0.0\"");
assert_modulo("false", "0.0");

trace("//0.0 % \"0.0\"");
assert_modulo(0.0, "0.0");

trace("//NaN % \"0.0\"");
assert_modulo(NaN, "0.0");

trace("//-0.0 % \"0.0\"");
assert_modulo(-0.0, "0.0");

trace("//Infinity % \"0.0\"");
assert_modulo(Infinity, "0.0");

trace("//1.0 % \"0.0\"");
assert_modulo(1.0, "0.0");

trace("//-1.0 % \"0.0\"");
assert_modulo(-1.0, "0.0");

trace("//0xFF1306 % \"0.0\"");
assert_modulo(0xFF1306, "0.0");

trace("//new Object() % \"0.0\"");
assert_modulo({}, "0.0");

trace("//\"0.0\" % \"0.0\"");
assert_modulo("0.0", "0.0");

trace("//\"NaN\" % \"0.0\"");
assert_modulo("NaN", "0.0");

trace("//\"-0.0\" % \"0.0\"");
assert_modulo("-0.0", "0.0");

trace("//\"Infinity\" % \"0.0\"");
assert_modulo("Infinity", "0.0");

trace("//\"1.0\" % \"0.0\"");
assert_modulo("1.0", "0.0");

trace("//\"-1.0\" % \"0.0\"");
assert_modulo("-1.0", "0.0");

trace("//\"0xFF1306\" % \"0.0\"");
assert_modulo("0xFF1306", "0.0");

trace("//true % \"NaN\"");
assert_modulo(true, "NaN");

trace("//false % \"NaN\"");
assert_modulo(false, "NaN");

trace("//null % \"NaN\"");
assert_modulo(null, "NaN");

trace("//undefined % \"NaN\"");
assert_modulo(undefined, "NaN");

trace("//\"\" % \"NaN\"");
assert_modulo("", "NaN");

trace("//\"str\" % \"NaN\"");
assert_modulo("str", "NaN");

trace("//\"true\" % \"NaN\"");
assert_modulo("true", "NaN");

trace("//\"false\" % \"NaN\"");
assert_modulo("false", "NaN");

trace("//0.0 % \"NaN\"");
assert_modulo(0.0, "NaN");

trace("//NaN % \"NaN\"");
assert_modulo(NaN, "NaN");

trace("//-0.0 % \"NaN\"");
assert_modulo(-0.0, "NaN");

trace("//Infinity % \"NaN\"");
assert_modulo(Infinity, "NaN");

trace("//1.0 % \"NaN\"");
assert_modulo(1.0, "NaN");

trace("//-1.0 % \"NaN\"");
assert_modulo(-1.0, "NaN");

trace("//0xFF1306 % \"NaN\"");
assert_modulo(0xFF1306, "NaN");

trace("//new Object() % \"NaN\"");
assert_modulo({}, "NaN");

trace("//\"0.0\" % \"NaN\"");
assert_modulo("0.0", "NaN");

trace("//\"NaN\" % \"NaN\"");
assert_modulo("NaN", "NaN");

trace("//\"-0.0\" % \"NaN\"");
assert_modulo("-0.0", "NaN");

trace("//\"Infinity\" % \"NaN\"");
assert_modulo("Infinity", "NaN");

trace("//\"1.0\" % \"NaN\"");
assert_modulo("1.0", "NaN");

trace("//\"-1.0\" % \"NaN\"");
assert_modulo("-1.0", "NaN");

trace("//\"0xFF1306\" % \"NaN\"");
assert_modulo("0xFF1306", "NaN");

trace("//true % \"-0.0\"");
assert_modulo(true, "-0.0");

trace("//false % \"-0.0\"");
assert_modulo(false, "-0.0");

trace("//null % \"-0.0\"");
assert_modulo(null, "-0.0");

trace("//undefined % \"-0.0\"");
assert_modulo(undefined, "-0.0");

trace("//\"\" % \"-0.0\"");
assert_modulo("", "-0.0");

trace("//\"str\" % \"-0.0\"");
assert_modulo("str", "-0.0");

trace("//\"true\" % \"-0.0\"");
assert_modulo("true", "-0.0");

trace("//\"false\" % \"-0.0\"");
assert_modulo("false", "-0.0");

trace("//0.0 % \"-0.0\"");
assert_modulo(0.0, "-0.0");

trace("//NaN % \"-0.0\"");
assert_modulo(NaN, "-0.0");

trace("//-0.0 % \"-0.0\"");
assert_modulo(-0.0, "-0.0");

trace("//Infinity % \"-0.0\"");
assert_modulo(Infinity, "-0.0");

trace("//1.0 % \"-0.0\"");
assert_modulo(1.0, "-0.0");

trace("//-1.0 % \"-0.0\"");
assert_modulo(-1.0, "-0.0");

trace("//0xFF1306 % \"-0.0\"");
assert_modulo(0xFF1306, "-0.0");

trace("//new Object() % \"-0.0\"");
assert_modulo({}, "-0.0");

trace("//\"0.0\" % \"-0.0\"");
assert_modulo("0.0", "-0.0");

trace("//\"NaN\" % \"-0.0\"");
assert_modulo("NaN", "-0.0");

trace("//\"-0.0\" % \"-0.0\"");
assert_modulo("-0.0", "-0.0");

trace("//\"Infinity\" % \"-0.0\"");
assert_modulo("Infinity", "-0.0");

trace("//\"1.0\" % \"-0.0\"");
assert_modulo("1.0", "-0.0");

trace("//\"-1.0\" % \"-0.0\"");
assert_modulo("-1.0", "-0.0");

trace("//\"0xFF1306\" % \"-0.0\"");
assert_modulo("0xFF1306", "-0.0");

trace("//true % \"Infinity\"");
assert_modulo(true, "Infinity");

trace("//false % \"Infinity\"");
assert_modulo(false, "Infinity");

trace("//null % \"Infinity\"");
assert_modulo(null, "Infinity");

trace("//undefined % \"Infinity\"");
assert_modulo(undefined, "Infinity");

trace("//\"\" % \"Infinity\"");
assert_modulo("", "Infinity");

trace("//\"str\" % \"Infinity\"");
assert_modulo("str", "Infinity");

trace("//\"true\" % \"Infinity\"");
assert_modulo("true", "Infinity");

trace("//\"false\" % \"Infinity\"");
assert_modulo("false", "Infinity");

trace("//0.0 % \"Infinity\"");
assert_modulo(0.0, "Infinity");

trace("//NaN % \"Infinity\"");
assert_modulo(NaN, "Infinity");

trace("//-0.0 % \"Infinity\"");
assert_modulo(-0.0, "Infinity");

trace("//Infinity % \"Infinity\"");
assert_modulo(Infinity, "Infinity");

trace("//1.0 % \"Infinity\"");
assert_modulo(1.0, "Infinity");

trace("//-1.0 % \"Infinity\"");
assert_modulo(-1.0, "Infinity");

trace("//0xFF1306 % \"Infinity\"");
assert_modulo(0xFF1306, "Infinity");

trace("//new Object() % \"Infinity\"");
assert_modulo({}, "Infinity");

trace("//\"0.0\" % \"Infinity\"");
assert_modulo("0.0", "Infinity");

trace("//\"NaN\" % \"Infinity\"");
assert_modulo("NaN", "Infinity");

trace("//\"-0.0\" % \"Infinity\"");
assert_modulo("-0.0", "Infinity");

trace("//\"Infinity\" % \"Infinity\"");
assert_modulo("Infinity", "Infinity");

trace("//\"1.0\" % \"Infinity\"");
assert_modulo("1.0", "Infinity");

trace("//\"-1.0\" % \"Infinity\"");
assert_modulo("-1.0", "Infinity");

trace("//\"0xFF1306\" % \"Infinity\"");
assert_modulo("0xFF1306", "Infinity");

trace("//true % \"1.0\"");
assert_modulo(true, "1.0");

trace("//false % \"1.0\"");
assert_modulo(false, "1.0");

trace("//null % \"1.0\"");
assert_modulo(null, "1.0");

trace("//undefined % \"1.0\"");
assert_modulo(undefined, "1.0");

trace("//\"\" % \"1.0\"");
assert_modulo("", "1.0");

trace("//\"str\" % \"1.0\"");
assert_modulo("str", "1.0");

trace("//\"true\" % \"1.0\"");
assert_modulo("true", "1.0");

trace("//\"false\" % \"1.0\"");
assert_modulo("false", "1.0");

trace("//0.0 % \"1.0\"");
assert_modulo(0.0, "1.0");

trace("//NaN % \"1.0\"");
assert_modulo(NaN, "1.0");

trace("//-0.0 % \"1.0\"");
assert_modulo(-0.0, "1.0");

trace("//Infinity % \"1.0\"");
assert_modulo(Infinity, "1.0");

trace("//1.0 % \"1.0\"");
assert_modulo(1.0, "1.0");

trace("//-1.0 % \"1.0\"");
assert_modulo(-1.0, "1.0");

trace("//0xFF1306 % \"1.0\"");
assert_modulo(0xFF1306, "1.0");

trace("//new Object() % \"1.0\"");
assert_modulo({}, "1.0");

trace("//\"0.0\" % \"1.0\"");
assert_modulo("0.0", "1.0");

trace("//\"NaN\" % \"1.0\"");
assert_modulo("NaN", "1.0");

trace("//\"-0.0\" % \"1.0\"");
assert_modulo("-0.0", "1.0");

trace("//\"Infinity\" % \"1.0\"");
assert_modulo("Infinity", "1.0");

trace("//\"1.0\" % \"1.0\"");
assert_modulo("1.0", "1.0");

trace("//\"-1.0\" % \"1.0\"");
assert_modulo("-1.0", "1.0");

trace("//\"0xFF1306\" % \"1.0\"");
assert_modulo("0xFF1306", "1.0");

trace("//true % \"-1.0\"");
assert_modulo(true, "-1.0");

trace("//false % \"-1.0\"");
assert_modulo(false, "-1.0");

trace("//null % \"-1.0\"");
assert_modulo(null, "-1.0");

trace("//undefined % \"-1.0\"");
assert_modulo(undefined, "-1.0");

trace("//\"\" % \"-1.0\"");
assert_modulo("", "-1.0");

trace("//\"str\" % \"-1.0\"");
assert_modulo("str", "-1.0");

trace("//\"true\" % \"-1.0\"");
assert_modulo("true", "-1.0");

trace("//\"false\" % \"-1.0\"");
assert_modulo("false", "-1.0");

trace("//0.0 % \"-1.0\"");
assert_modulo(0.0, "-1.0");

trace("//NaN % \"-1.0\"");
assert_modulo(NaN, "-1.0");

trace("//-0.0 % \"-1.0\"");
assert_modulo(-0.0, "-1.0");

trace("//Infinity % \"-1.0\"");
assert_modulo(Infinity, "-1.0");

trace("//1.0 % \"-1.0\"");
assert_modulo(1.0, "-1.0");

trace("//-1.0 % \"-1.0\"");
assert_modulo(-1.0, "-1.0");

trace("//0xFF1306 % \"-1.0\"");
assert_modulo(0xFF1306, "-1.0");

trace("//new Object() % \"-1.0\"");
assert_modulo({}, "-1.0");

trace("//\"0.0\" % \"-1.0\"");
assert_modulo("0.0", "-1.0");

trace("//\"NaN\" % \"-1.0\"");
assert_modulo("NaN", "-1.0");

trace("//\"-0.0\" % \"-1.0\"");
assert_modulo("-0.0", "-1.0");

trace("//\"Infinity\" % \"-1.0\"");
assert_modulo("Infinity", "-1.0");

trace("//\"1.0\" % \"-1.0\"");
assert_modulo("1.0", "-1.0");

trace("//\"-1.0\" % \"-1.0\"");
assert_modulo("-1.0", "-1.0");

trace("//\"0xFF1306\" % \"-1.0\"");
assert_modulo("0xFF1306", "-1.0");

trace("//true % \"0xFF1306\"");
assert_modulo(true, "0xFF1306");

trace("//false % \"0xFF1306\"");
assert_modulo(false, "0xFF1306");

trace("//null % \"0xFF1306\"");
assert_modulo(null, "0xFF1306");

trace("//undefined % \"0xFF1306\"");
assert_modulo(undefined, "0xFF1306");

trace("//\"\" % \"0xFF1306\"");
assert_modulo("", "0xFF1306");

trace("//\"str\" % \"0xFF1306\"");
assert_modulo("str", "0xFF1306");

trace("//\"true\" % \"0xFF1306\"");
assert_modulo("true", "0xFF1306");

trace("//\"false\" % \"0xFF1306\"");
assert_modulo("false", "0xFF1306");

trace("//0.0 % \"0xFF1306\"");
assert_modulo(0.0, "0xFF1306");

trace("//NaN % \"0xFF1306\"");
assert_modulo(NaN, "0xFF1306");

trace("//-0.0 % \"0xFF1306\"");
assert_modulo(-0.0, "0xFF1306");

trace("//Infinity % \"0xFF1306\"");
assert_modulo(Infinity, "0xFF1306");

trace("//1.0 % \"0xFF1306\"");
assert_modulo(1.0, "0xFF1306");

trace("//-1.0 % \"0xFF1306\"");
assert_modulo(-1.0, "0xFF1306");

trace("//0xFF1306 % \"0xFF1306\"");
assert_modulo(0xFF1306, "0xFF1306");

trace("//new Object() % \"0xFF1306\"");
assert_modulo({}, "0xFF1306");

trace("//\"0.0\" % \"0xFF1306\"");
assert_modulo("0.0", "0xFF1306");

trace("//\"NaN\" % \"0xFF1306\"");
assert_modulo("NaN", "0xFF1306");

trace("//\"-0.0\" % \"0xFF1306\"");
assert_modulo("-0.0", "0xFF1306");

trace("//\"Infinity\" % \"0xFF1306\"");
assert_modulo("Infinity", "0xFF1306");

trace("//\"1.0\" % \"0xFF1306\"");
assert_modulo("1.0", "0xFF1306");

trace("//\"-1.0\" % \"0xFF1306\"");
assert_modulo("-1.0", "0xFF1306");

trace("//\"0xFF1306\" % \"0xFF1306\"");
assert_modulo("0xFF1306", "0xFF1306");