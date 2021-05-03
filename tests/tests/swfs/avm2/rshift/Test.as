package {
	public class Test {
	}
}

function assert_rshift(val1, val2) {
	trace(val1 >> val2);
}

trace("//true >> true");
assert_rshift(true, true);

trace("//false >> true");
assert_rshift(false, true);

trace("//null >> true");
assert_rshift(null, true);

trace("//undefined >> true");
assert_rshift(undefined, true);

trace("//\"\" >> true");
assert_rshift("", true);

trace("//\"str\" >> true");
assert_rshift("str", true);

trace("//\"true\" >> true");
assert_rshift("true", true);

trace("//\"false\" >> true");
assert_rshift("false", true);

trace("//0.0 >> true");
assert_rshift(0.0, true);

trace("//NaN >> true");
assert_rshift(NaN, true);

trace("//-0.0 >> true");
assert_rshift(-0.0, true);

trace("//Infinity >> true");
assert_rshift(Infinity, true);

trace("//1.0 >> true");
assert_rshift(1.0, true);

trace("//-1.0 >> true");
assert_rshift(-1.0, true);

trace("//0xFF1306 >> true");
assert_rshift(0xFF1306, true);

trace("//new Object() >> true");
assert_rshift({}, true);

trace("//\"0.0\" >> true");
assert_rshift("0.0", true);

trace("//\"NaN\" >> true");
assert_rshift("NaN", true);

trace("//\"-0.0\" >> true");
assert_rshift("-0.0", true);

trace("//\"Infinity\" >> true");
assert_rshift("Infinity", true);

trace("//\"1.0\" >> true");
assert_rshift("1.0", true);

trace("//\"-1.0\" >> true");
assert_rshift("-1.0", true);

trace("//\"0xFF1306\" >> true");
assert_rshift("0xFF1306", true);

trace("//true >> false");
assert_rshift(true, false);

trace("//false >> false");
assert_rshift(false, false);

trace("//null >> false");
assert_rshift(null, false);

trace("//undefined >> false");
assert_rshift(undefined, false);

trace("//\"\" >> false");
assert_rshift("", false);

trace("//\"str\" >> false");
assert_rshift("str", false);

trace("//\"true\" >> false");
assert_rshift("true", false);

trace("//\"false\" >> false");
assert_rshift("false", false);

trace("//0.0 >> false");
assert_rshift(0.0, false);

trace("//NaN >> false");
assert_rshift(NaN, false);

trace("//-0.0 >> false");
assert_rshift(-0.0, false);

trace("//Infinity >> false");
assert_rshift(Infinity, false);

trace("//1.0 >> false");
assert_rshift(1.0, false);

trace("//-1.0 >> false");
assert_rshift(-1.0, false);

trace("//0xFF1306 >> false");
assert_rshift(0xFF1306, false);

trace("//new Object() >> false");
assert_rshift({}, false);

trace("//\"0.0\" >> false");
assert_rshift("0.0", false);

trace("//\"NaN\" >> false");
assert_rshift("NaN", false);

trace("//\"-0.0\" >> false");
assert_rshift("-0.0", false);

trace("//\"Infinity\" >> false");
assert_rshift("Infinity", false);

trace("//\"1.0\" >> false");
assert_rshift("1.0", false);

trace("//\"-1.0\" >> false");
assert_rshift("-1.0", false);

trace("//\"0xFF1306\" >> false");
assert_rshift("0xFF1306", false);
trace("//true >> null");
assert_rshift(true, null);

trace("//false >> null");
assert_rshift(false, null);

trace("//null >> null");
assert_rshift(null, null);

trace("//undefined >> null");
assert_rshift(undefined, null);

trace("//\"\" >> null");
assert_rshift("", null);

trace("//\"str\" >> null");
assert_rshift("str", null);

trace("//\"true\" >> null");
assert_rshift("true", null);

trace("//\"false\" >> null");
assert_rshift("false", null);

trace("//0.0 >> null");
assert_rshift(0.0, null);

trace("//NaN >> null");
assert_rshift(NaN, null);

trace("//-0.0 >> null");
assert_rshift(-0.0, null);

trace("//Infinity >> null");
assert_rshift(Infinity, null);

trace("//1.0 >> null");
assert_rshift(1.0, null);

trace("//-1.0 >> null");
assert_rshift(-1.0, null);

trace("//0xFF1306 >> null");
assert_rshift(0xFF1306, null);

trace("//new Object() >> null");
assert_rshift({}, null);

trace("//\"0.0\" >> null");
assert_rshift("0.0", null);

trace("//\"NaN\" >> null");
assert_rshift("NaN", null);

trace("//\"-0.0\" >> null");
assert_rshift("-0.0", null);

trace("//\"Infinity\" >> null");
assert_rshift("Infinity", null);

trace("//\"1.0\" >> null");
assert_rshift("1.0", null);

trace("//\"-1.0\" >> null");
assert_rshift("-1.0", null);

trace("//\"0xFF1306\" >> null");
assert_rshift("0xFF1306", null);

trace("//true >> undefined");
assert_rshift(true, undefined);

trace("//false >> undefined");
assert_rshift(false, undefined);

trace("//null >> undefined");
assert_rshift(null, undefined);

trace("//undefined >> undefined");
assert_rshift(undefined, undefined);

trace("//\"\" >> undefined");
assert_rshift("", undefined);

trace("//\"str\" >> undefined");
assert_rshift("str", undefined);

trace("//\"true\" >> undefined");
assert_rshift("true", undefined);

trace("//\"false\" >> undefined");
assert_rshift("false", undefined);

trace("//0.0 >> undefined");
assert_rshift(0.0, undefined);

trace("//NaN >> undefined");
assert_rshift(NaN, undefined);

trace("//-0.0 >> undefined");
assert_rshift(-0.0, undefined);

trace("//Infinity >> undefined");
assert_rshift(Infinity, undefined);

trace("//1.0 >> undefined");
assert_rshift(1.0, undefined);

trace("//-1.0 >> undefined");
assert_rshift(-1.0, undefined);

trace("//0xFF1306 >> undefined");
assert_rshift(0xFF1306, undefined);

trace("//new Object() >> undefined");
assert_rshift({}, undefined);

trace("//\"0.0\" >> undefined");
assert_rshift("0.0", undefined);

trace("//\"NaN\" >> undefined");
assert_rshift("NaN", undefined);

trace("//\"-0.0\" >> undefined");
assert_rshift("-0.0", undefined);

trace("//\"Infinity\" >> undefined");
assert_rshift("Infinity", undefined);

trace("//\"1.0\" >> undefined");
assert_rshift("1.0", undefined);

trace("//\"-1.0\" >> undefined");
assert_rshift("-1.0", undefined);

trace("//\"0xFF1306\" >> undefined");
assert_rshift("0xFF1306", undefined);

trace("//true >> \"\"");
assert_rshift(true, "");

trace("//false >> \"\"");
assert_rshift(false, "");

trace("//null >> \"\"");
assert_rshift(null, "");

trace("//undefined >> \"\"");
assert_rshift(undefined, "");

trace("//\"\" >> \"\"");
assert_rshift("", "");

trace("//\"str\" >> \"\"");
assert_rshift("str", "");

trace("//\"true\" >> \"\"");
assert_rshift("true", "");

trace("//\"false\" >> \"\"");
assert_rshift("false", "");

trace("//0.0 >> \"\"");
assert_rshift(0.0, "");

trace("//NaN >> \"\"");
assert_rshift(NaN, "");

trace("//-0.0 >> \"\"");
assert_rshift(-0.0, "");

trace("//Infinity >> \"\"");
assert_rshift(Infinity, "");

trace("//1.0 >> \"\"");
assert_rshift(1.0, "");

trace("//-1.0 >> \"\"");
assert_rshift(-1.0, "");

trace("//0xFF1306 >> \"\"");
assert_rshift(0xFF1306, "");

trace("//new Object() >> \"\"");
assert_rshift({}, "");

trace("//\"0.0\" >> \"\"");
assert_rshift("0.0", "");

trace("//\"NaN\" >> \"\"");
assert_rshift("NaN", "");

trace("//\"-0.0\" >> \"\"");
assert_rshift("-0.0", "");

trace("//\"Infinity\" >> \"\"");
assert_rshift("Infinity", "");

trace("//\"1.0\" >> \"\"");
assert_rshift("1.0", "");

trace("//\"-1.0\" >> \"\"");
assert_rshift("-1.0", "");

trace("//\"0xFF1306\" >> \"\"");
assert_rshift("0xFF1306", "");

trace("//true >> \"str\"");
assert_rshift(true, "str");

trace("//false >> \"str\"");
assert_rshift(false, "str");

trace("//null >> \"str\"");
assert_rshift(null, "str");

trace("//undefined >> \"str\"");
assert_rshift(undefined, "str");

trace("//\"\" >> \"str\"");
assert_rshift("", "str");

trace("//\"str\" >> \"str\"");
assert_rshift("str", "str");

trace("//\"true\" >> \"str\"");
assert_rshift("true", "str");

trace("//\"false\" >> \"str\"");
assert_rshift("false", "str");

trace("//0.0 >> \"str\"");
assert_rshift(0.0, "str");

trace("//NaN >> \"str\"");
assert_rshift(NaN, "str");

trace("//-0.0 >> \"str\"");
assert_rshift(-0.0, "str");

trace("//Infinity >> \"str\"");
assert_rshift(Infinity, "str");

trace("//1.0 >> \"str\"");
assert_rshift(1.0, "str");

trace("//-1.0 >> \"str\"");
assert_rshift(-1.0, "str");

trace("//0xFF1306 >> \"str\"");
assert_rshift(0xFF1306, "str");

trace("//new Object() >> \"str\"");
assert_rshift({}, "str");

trace("//\"0.0\" >> \"str\"");
assert_rshift("0.0", "str");

trace("//\"NaN\" >> \"str\"");
assert_rshift("NaN", "str");

trace("//\"-0.0\" >> \"str\"");
assert_rshift("-0.0", "str");

trace("//\"Infinity\" >> \"str\"");
assert_rshift("Infinity", "str");

trace("//\"1.0\" >> \"str\"");
assert_rshift("1.0", "str");

trace("//\"-1.0\" >> \"str\"");
assert_rshift("-1.0", "str");

trace("//\"0xFF1306\" >> \"str\"");
assert_rshift("0xFF1306", "str");

trace("//true >> \"true\"");
assert_rshift(true, "true");

trace("//false >> \"true\"");
assert_rshift(false, "true");

trace("//null >> \"true\"");
assert_rshift(null, "true");

trace("//undefined >> \"true\"");
assert_rshift(undefined, "true");

trace("//\"\" >> \"true\"");
assert_rshift("", "true");

trace("//\"str\" >> \"true\"");
assert_rshift("str", "true");

trace("//\"true\" >> \"true\"");
assert_rshift("true", "true");

trace("//\"false\" >> \"true\"");
assert_rshift("false", "true");

trace("//0.0 >> \"true\"");
assert_rshift(0.0, "true");

trace("//NaN >> \"true\"");
assert_rshift(NaN, "true");

trace("//-0.0 >> \"true\"");
assert_rshift(-0.0, "true");

trace("//Infinity >> \"true\"");
assert_rshift(Infinity, "true");

trace("//1.0 >> \"true\"");
assert_rshift(1.0, "true");

trace("//-1.0 >> \"true\"");
assert_rshift(-1.0, "true");

trace("//0xFF1306 >> \"true\"");
assert_rshift(0xFF1306, "true");

trace("//new Object() >> \"true\"");
assert_rshift({}, "true");

trace("//\"0.0\" >> \"true\"");
assert_rshift("0.0", "true");

trace("//\"NaN\" >> \"true\"");
assert_rshift("NaN", "true");

trace("//\"-0.0\" >> \"true\"");
assert_rshift("-0.0", "true");

trace("//\"Infinity\" >> \"true\"");
assert_rshift("Infinity", "true");

trace("//\"1.0\" >> \"true\"");
assert_rshift("1.0", "true");

trace("//\"-1.0\" >> \"true\"");
assert_rshift("-1.0", "true");

trace("//\"0xFF1306\" >> \"true\"");
assert_rshift("0xFF1306", "true");

trace("//true >> \"false\"");
assert_rshift(true, "false");

trace("//false >> \"false\"");
assert_rshift(false, "false");

trace("//null >> \"false\"");
assert_rshift(null, "false");

trace("//undefined >> \"false\"");
assert_rshift(undefined, "false");

trace("//\"\" >> \"false\"");
assert_rshift("", "false");

trace("//\"str\" >> \"false\"");
assert_rshift("str", "false");

trace("//\"true\" >> \"false\"");
assert_rshift("true", "false");

trace("//\"false\" >> \"false\"");
assert_rshift("false", "false");

trace("//0.0 >> \"false\"");
assert_rshift(0.0, "false");

trace("//NaN >> \"false\"");
assert_rshift(NaN, "false");

trace("//-0.0 >> \"false\"");
assert_rshift(-0.0, "false");

trace("//Infinity >> \"false\"");
assert_rshift(Infinity, "false");

trace("//1.0 >> \"false\"");
assert_rshift(1.0, "false");

trace("//-1.0 >> \"false\"");
assert_rshift(-1.0, "false");

trace("//0xFF1306 >> \"false\"");
assert_rshift(0xFF1306, "false");

trace("//new Object() >> \"false\"");
assert_rshift({}, "false");

trace("//\"0.0\" >> \"false\"");
assert_rshift("0.0", "false");

trace("//\"NaN\" >> \"false\"");
assert_rshift("NaN", "false");

trace("//\"-0.0\" >> \"false\"");
assert_rshift("-0.0", "false");

trace("//\"Infinity\" >> \"false\"");
assert_rshift("Infinity", "false");

trace("//\"1.0\" >> \"false\"");
assert_rshift("1.0", "false");

trace("//\"-1.0\" >> \"false\"");
assert_rshift("-1.0", "false");

trace("//\"0xFF1306\" >> \"false\"");
assert_rshift("0xFF1306", "false");

trace("//true >> 0.0");
assert_rshift(true, 0.0);

trace("//false >> 0.0");
assert_rshift(false, 0.0);

trace("//null >> 0.0");
assert_rshift(null, 0.0);

trace("//undefined >> 0.0");
assert_rshift(undefined, 0.0);

trace("//\"\" >> 0.0");
assert_rshift("", 0.0);

trace("//\"str\" >> 0.0");
assert_rshift("str", 0.0);

trace("//\"true\" >> 0.0");
assert_rshift("true", 0.0);

trace("//\"false\" >> 0.0");
assert_rshift("false", 0.0);

trace("//0.0 >> 0.0");
assert_rshift(0.0, 0.0);

trace("//NaN >> 0.0");
assert_rshift(NaN, 0.0);

trace("//-0.0 >> 0.0");
assert_rshift(-0.0, 0.0);

trace("//Infinity >> 0.0");
assert_rshift(Infinity, 0.0);

trace("//1.0 >> 0.0");
assert_rshift(1.0, 0.0);

trace("//-1.0 >> 0.0");
assert_rshift(-1.0, 0.0);

trace("//0xFF1306 >> 0.0");
assert_rshift(0xFF1306, 0.0);

trace("//new Object() >> 0.0");
assert_rshift({}, 0.0);

trace("//\"0.0\" >> 0.0");
assert_rshift("0.0", 0.0);

trace("//\"NaN\" >> 0.0");
assert_rshift("NaN", 0.0);

trace("//\"-0.0\" >> 0.0");
assert_rshift("-0.0", 0.0);

trace("//\"Infinity\" >> 0.0");
assert_rshift("Infinity", 0.0);

trace("//\"1.0\" >> 0.0");
assert_rshift("1.0", 0.0);

trace("//\"-1.0\" >> 0.0");
assert_rshift("-1.0", 0.0);

trace("//\"0xFF1306\" >> 0.0");
assert_rshift("0xFF1306", 0.0);

trace("//true >> NaN");
assert_rshift(true, NaN);

trace("//false >> NaN");
assert_rshift(false, NaN);

trace("//null >> NaN");
assert_rshift(null, NaN);

trace("//undefined >> NaN");
assert_rshift(undefined, NaN);

trace("//\"\" >> NaN");
assert_rshift("", NaN);

trace("//\"str\" >> NaN");
assert_rshift("str", NaN);

trace("//\"true\" >> NaN");
assert_rshift("true", NaN);

trace("//\"false\" >> NaN");
assert_rshift("false", NaN);

trace("//0.0 >> NaN");
assert_rshift(0.0, NaN);

trace("//NaN >> NaN");
assert_rshift(NaN, NaN);

trace("//-0.0 >> NaN");
assert_rshift(-0.0, NaN);

trace("//Infinity >> NaN");
assert_rshift(Infinity, NaN);

trace("//1.0 >> NaN");
assert_rshift(1.0, NaN);

trace("//-1.0 >> NaN");
assert_rshift(-1.0, NaN);

trace("//0xFF1306 >> NaN");
assert_rshift(0xFF1306, NaN);

trace("//new Object() >> NaN");
assert_rshift({}, NaN);

trace("//\"0.0\" >> NaN");
assert_rshift("0.0", NaN);

trace("//\"NaN\" >> NaN");
assert_rshift("NaN", NaN);

trace("//\"-0.0\" >> NaN");
assert_rshift("-0.0", NaN);

trace("//\"Infinity\" >> NaN");
assert_rshift("Infinity", NaN);

trace("//\"1.0\" >> NaN");
assert_rshift("1.0", NaN);

trace("//\"-1.0\" >> NaN");
assert_rshift("-1.0", NaN);

trace("//\"0xFF1306\" >> NaN");
assert_rshift("0xFF1306", NaN);

trace("//true >> -0.0");
assert_rshift(true, -0.0);

trace("//false >> -0.0");
assert_rshift(false, -0.0);

trace("//null >> -0.0");
assert_rshift(null, -0.0);

trace("//undefined >> -0.0");
assert_rshift(undefined, -0.0);

trace("//\"\" >> -0.0");
assert_rshift("", -0.0);

trace("//\"str\" >> -0.0");
assert_rshift("str", -0.0);

trace("//\"true\" >> -0.0");
assert_rshift("true", -0.0);

trace("//\"false\" >> -0.0");
assert_rshift("false", -0.0);

trace("//0.0 >> -0.0");
assert_rshift(0.0, -0.0);

trace("//NaN >> -0.0");
assert_rshift(NaN, -0.0);

trace("//-0.0 >> -0.0");
assert_rshift(-0.0, -0.0);

trace("//Infinity >> -0.0");
assert_rshift(Infinity, -0.0);

trace("//1.0 >> -0.0");
assert_rshift(1.0, -0.0);

trace("//-1.0 >> -0.0");
assert_rshift(-1.0, -0.0);

trace("//0xFF1306 >> -0.0");
assert_rshift(0xFF1306, -0.0);

trace("//new Object() >> -0.0");
assert_rshift({}, -0.0);

trace("//\"0.0\" >> -0.0");
assert_rshift("0.0", -0.0);

trace("//\"NaN\" >> -0.0");
assert_rshift("NaN", -0.0);

trace("//\"-0.0\" >> -0.0");
assert_rshift("-0.0", -0.0);

trace("//\"Infinity\" >> -0.0");
assert_rshift("Infinity", -0.0);

trace("//\"1.0\" >> -0.0");
assert_rshift("1.0", -0.0);

trace("//\"-1.0\" >> -0.0");
assert_rshift("-1.0", -0.0);

trace("//\"0xFF1306\" >> -0.0");
assert_rshift("0xFF1306", -0.0);

trace("//true >> Infinity");
assert_rshift(true, Infinity);

trace("//false >> Infinity");
assert_rshift(false, Infinity);

trace("//null >> Infinity");
assert_rshift(null, Infinity);

trace("//undefined >> Infinity");
assert_rshift(undefined, Infinity);

trace("//\"\" >> Infinity");
assert_rshift("", Infinity);

trace("//\"str\" >> Infinity");
assert_rshift("str", Infinity);

trace("//\"true\" >> Infinity");
assert_rshift("true", Infinity);

trace("//\"false\" >> Infinity");
assert_rshift("false", Infinity);

trace("//0.0 >> Infinity");
assert_rshift(0.0, Infinity);

trace("//NaN >> Infinity");
assert_rshift(NaN, Infinity);

trace("//-0.0 >> Infinity");
assert_rshift(-0.0, Infinity);

trace("//Infinity >> Infinity");
assert_rshift(Infinity, Infinity);

trace("//1.0 >> Infinity");
assert_rshift(1.0, Infinity);

trace("//-1.0 >> Infinity");
assert_rshift(-1.0, Infinity);

trace("//0xFF1306 >> Infinity");
assert_rshift(0xFF1306, Infinity);

trace("//new Object() >> Infinity");
assert_rshift({}, Infinity);

trace("//\"0.0\" >> Infinity");
assert_rshift("0.0", Infinity);

trace("//\"NaN\" >> Infinity");
assert_rshift("NaN", Infinity);

trace("//\"-0.0\" >> Infinity");
assert_rshift("-0.0", Infinity);

trace("//\"Infinity\" >> Infinity");
assert_rshift("Infinity", Infinity);

trace("//\"1.0\" >> Infinity");
assert_rshift("1.0", Infinity);

trace("//\"-1.0\" >> Infinity");
assert_rshift("-1.0", Infinity);

trace("//\"0xFF1306\" >> Infinity");
assert_rshift("0xFF1306", Infinity);

trace("//true >> 1.0");
assert_rshift(true, 1.0);

trace("//false >> 1.0");
assert_rshift(false, 1.0);

trace("//null >> 1.0");
assert_rshift(null, 1.0);

trace("//undefined >> 1.0");
assert_rshift(undefined, 1.0);

trace("//\"\" >> 1.0");
assert_rshift("", 1.0);

trace("//\"str\" >> 1.0");
assert_rshift("str", 1.0);

trace("//\"true\" >> 1.0");
assert_rshift("true", 1.0);

trace("//\"false\" >> 1.0");
assert_rshift("false", 1.0);

trace("//0.0 >> 1.0");
assert_rshift(0.0, 1.0);

trace("//NaN >> 1.0");
assert_rshift(NaN, 1.0);

trace("//-0.0 >> 1.0");
assert_rshift(-0.0, 1.0);

trace("//Infinity >> 1.0");
assert_rshift(Infinity, 1.0);

trace("//1.0 >> 1.0");
assert_rshift(1.0, 1.0);

trace("//-1.0 >> 1.0");
assert_rshift(-1.0, 1.0);

trace("//0xFF1306 >> 1.0");
assert_rshift(0xFF1306, 1.0);

trace("//new Object() >> 1.0");
assert_rshift({}, 1.0);

trace("//\"0.0\" >> 1.0");
assert_rshift("0.0", 1.0);

trace("//\"NaN\" >> 1.0");
assert_rshift("NaN", 1.0);

trace("//\"-0.0\" >> 1.0");
assert_rshift("-0.0", 1.0);

trace("//\"Infinity\" >> 1.0");
assert_rshift("Infinity", 1.0);

trace("//\"1.0\" >> 1.0");
assert_rshift("1.0", 1.0);

trace("//\"-1.0\" >> 1.0");
assert_rshift("-1.0", 1.0);

trace("//\"0xFF1306\" >> 1.0");
assert_rshift("0xFF1306", 1.0);

trace("//true >> -1.0");
assert_rshift(true, -1.0);

trace("//false >> -1.0");
assert_rshift(false, -1.0);

trace("//null >> -1.0");
assert_rshift(null, -1.0);

trace("//undefined >> -1.0");
assert_rshift(undefined, -1.0);

trace("//\"\" >> -1.0");
assert_rshift("", -1.0);

trace("//\"str\" >> -1.0");
assert_rshift("str", -1.0);

trace("//\"true\" >> -1.0");
assert_rshift("true", -1.0);

trace("//\"false\" >> -1.0");
assert_rshift("false", -1.0);

trace("//0.0 >> -1.0");
assert_rshift(0.0, -1.0);

trace("//NaN >> -1.0");
assert_rshift(NaN, -1.0);

trace("//-0.0 >> -1.0");
assert_rshift(-0.0, -1.0);

trace("//Infinity >> -1.0");
assert_rshift(Infinity, -1.0);

trace("//1.0 >> -1.0");
assert_rshift(1.0, -1.0);

trace("//-1.0 >> -1.0");
assert_rshift(-1.0, -1.0);

trace("//0xFF1306 >> -1.0");
assert_rshift(0xFF1306, -1.0);

trace("//new Object() >> -1.0");
assert_rshift({}, -1.0);

trace("//\"0.0\" >> -1.0");
assert_rshift("0.0", -1.0);

trace("//\"NaN\" >> -1.0");
assert_rshift("NaN", -1.0);

trace("//\"-0.0\" >> -1.0");
assert_rshift("-0.0", -1.0);

trace("//\"Infinity\" >> -1.0");
assert_rshift("Infinity", -1.0);

trace("//\"1.0\" >> -1.0");
assert_rshift("1.0", -1.0);

trace("//\"-1.0\" >> -1.0");
assert_rshift("-1.0", -1.0);

trace("//\"0xFF1306\" >> -1.0");
assert_rshift("0xFF1306", -1.0);

trace("//true >> 0xFF1306");
assert_rshift(true, 0xFF1306);

trace("//false >> 0xFF1306");
assert_rshift(false, 0xFF1306);

trace("//null >> 0xFF1306");
assert_rshift(null, 0xFF1306);

trace("//undefined >> 0xFF1306");
assert_rshift(undefined, 0xFF1306);

trace("//\"\" >> 0xFF1306");
assert_rshift("", 0xFF1306);

trace("//\"str\" >> 0xFF1306");
assert_rshift("str", 0xFF1306);

trace("//\"true\" >> 0xFF1306");
assert_rshift("true", 0xFF1306);

trace("//\"false\" >> 0xFF1306");
assert_rshift("false", 0xFF1306);

trace("//0.0 >> 0xFF1306");
assert_rshift(0.0, 0xFF1306);

trace("//NaN >> 0xFF1306");
assert_rshift(NaN, 0xFF1306);

trace("//-0.0 >> 0xFF1306");
assert_rshift(-0.0, 0xFF1306);

trace("//Infinity >> 0xFF1306");
assert_rshift(Infinity, 0xFF1306);

trace("//1.0 >> 0xFF1306");
assert_rshift(1.0, 0xFF1306);

trace("//-1.0 >> 0xFF1306");
assert_rshift(-1.0, 0xFF1306);

trace("//0xFF1306 >> 0xFF1306");
assert_rshift(0xFF1306, 0xFF1306);

trace("//new Object() >> 0xFF1306");
assert_rshift({}, 0xFF1306);

trace("//\"0.0\" >> 0xFF1306");
assert_rshift("0.0", 0xFF1306);

trace("//\"NaN\" >> 0xFF1306");
assert_rshift("NaN", 0xFF1306);

trace("//\"-0.0\" >> 0xFF1306");
assert_rshift("-0.0", 0xFF1306);

trace("//\"Infinity\" >> 0xFF1306");
assert_rshift("Infinity", 0xFF1306);

trace("//\"1.0\" >> 0xFF1306");
assert_rshift("1.0", 0xFF1306);

trace("//\"-1.0\" >> 0xFF1306");
assert_rshift("-1.0", 0xFF1306);

trace("//\"0xFF1306\" >> 0xFF1306");
assert_rshift("0xFF1306", 0xFF1306);

trace("//true >> new Object()");
assert_rshift(true, {});

trace("//false >> new Object()");
assert_rshift(false, {});

trace("//null >> new Object()");
assert_rshift(null, {});

trace("//undefined >> new Object()");
assert_rshift(undefined, {});

trace("//\"\" >> new Object()");
assert_rshift("", {});

trace("//\"str\" >> new Object()");
assert_rshift("str", {});

trace("//\"true\" >> new Object()");
assert_rshift("true", {});

trace("//\"false\" >> new Object()");
assert_rshift("false", {});

trace("//0.0 >> new Object()");
assert_rshift(0.0, {});

trace("//NaN >> new Object()");
assert_rshift(NaN, {});

trace("//-0.0 >> new Object()");
assert_rshift(-0.0, {});

trace("//Infinity >> new Object()");
assert_rshift(Infinity, {});

trace("//1.0 >> new Object()");
assert_rshift(1.0, {});

trace("//-1.0 >> new Object()");
assert_rshift(-1.0, {});

trace("//0xFF1306 >> new Object()");
assert_rshift(0xFF1306, {});

trace("//new Object() >> new Object()");
assert_rshift({}, {});

trace("//\"0.0\" >> new Object()");
assert_rshift("0.0", {});

trace("//\"NaN\" >> new Object()");
assert_rshift("NaN", {});

trace("//\"-0.0\" >> new Object()");
assert_rshift("-0.0", {});

trace("//\"Infinity\" >> new Object()");
assert_rshift("Infinity", {});

trace("//\"1.0\" >> new Object()");
assert_rshift("1.0", {});

trace("//\"-1.0\" >> new Object()");
assert_rshift("-1.0", {});

trace("//\"0xFF1306\" >> new Object()");
assert_rshift("0xFF1306", {});

trace("//true >> \"0.0\"");
assert_rshift(true, "0.0");

trace("//false >> \"0.0\"");
assert_rshift(false, "0.0");

trace("//null >> \"0.0\"");
assert_rshift(null, "0.0");

trace("//undefined >> \"0.0\"");
assert_rshift(undefined, "0.0");

trace("//\"\" >> \"0.0\"");
assert_rshift("", "0.0");

trace("//\"str\" >> \"0.0\"");
assert_rshift("str", "0.0");

trace("//\"true\" >> \"0.0\"");
assert_rshift("true", "0.0");

trace("//\"false\" >> \"0.0\"");
assert_rshift("false", "0.0");

trace("//0.0 >> \"0.0\"");
assert_rshift(0.0, "0.0");

trace("//NaN >> \"0.0\"");
assert_rshift(NaN, "0.0");

trace("//-0.0 >> \"0.0\"");
assert_rshift(-0.0, "0.0");

trace("//Infinity >> \"0.0\"");
assert_rshift(Infinity, "0.0");

trace("//1.0 >> \"0.0\"");
assert_rshift(1.0, "0.0");

trace("//-1.0 >> \"0.0\"");
assert_rshift(-1.0, "0.0");

trace("//0xFF1306 >> \"0.0\"");
assert_rshift(0xFF1306, "0.0");

trace("//new Object() >> \"0.0\"");
assert_rshift({}, "0.0");

trace("//\"0.0\" >> \"0.0\"");
assert_rshift("0.0", "0.0");

trace("//\"NaN\" >> \"0.0\"");
assert_rshift("NaN", "0.0");

trace("//\"-0.0\" >> \"0.0\"");
assert_rshift("-0.0", "0.0");

trace("//\"Infinity\" >> \"0.0\"");
assert_rshift("Infinity", "0.0");

trace("//\"1.0\" >> \"0.0\"");
assert_rshift("1.0", "0.0");

trace("//\"-1.0\" >> \"0.0\"");
assert_rshift("-1.0", "0.0");

trace("//\"0xFF1306\" >> \"0.0\"");
assert_rshift("0xFF1306", "0.0");

trace("//true >> \"NaN\"");
assert_rshift(true, "NaN");

trace("//false >> \"NaN\"");
assert_rshift(false, "NaN");

trace("//null >> \"NaN\"");
assert_rshift(null, "NaN");

trace("//undefined >> \"NaN\"");
assert_rshift(undefined, "NaN");

trace("//\"\" >> \"NaN\"");
assert_rshift("", "NaN");

trace("//\"str\" >> \"NaN\"");
assert_rshift("str", "NaN");

trace("//\"true\" >> \"NaN\"");
assert_rshift("true", "NaN");

trace("//\"false\" >> \"NaN\"");
assert_rshift("false", "NaN");

trace("//0.0 >> \"NaN\"");
assert_rshift(0.0, "NaN");

trace("//NaN >> \"NaN\"");
assert_rshift(NaN, "NaN");

trace("//-0.0 >> \"NaN\"");
assert_rshift(-0.0, "NaN");

trace("//Infinity >> \"NaN\"");
assert_rshift(Infinity, "NaN");

trace("//1.0 >> \"NaN\"");
assert_rshift(1.0, "NaN");

trace("//-1.0 >> \"NaN\"");
assert_rshift(-1.0, "NaN");

trace("//0xFF1306 >> \"NaN\"");
assert_rshift(0xFF1306, "NaN");

trace("//new Object() >> \"NaN\"");
assert_rshift({}, "NaN");

trace("//\"0.0\" >> \"NaN\"");
assert_rshift("0.0", "NaN");

trace("//\"NaN\" >> \"NaN\"");
assert_rshift("NaN", "NaN");

trace("//\"-0.0\" >> \"NaN\"");
assert_rshift("-0.0", "NaN");

trace("//\"Infinity\" >> \"NaN\"");
assert_rshift("Infinity", "NaN");

trace("//\"1.0\" >> \"NaN\"");
assert_rshift("1.0", "NaN");

trace("//\"-1.0\" >> \"NaN\"");
assert_rshift("-1.0", "NaN");

trace("//\"0xFF1306\" >> \"NaN\"");
assert_rshift("0xFF1306", "NaN");

trace("//true >> \"-0.0\"");
assert_rshift(true, "-0.0");

trace("//false >> \"-0.0\"");
assert_rshift(false, "-0.0");

trace("//null >> \"-0.0\"");
assert_rshift(null, "-0.0");

trace("//undefined >> \"-0.0\"");
assert_rshift(undefined, "-0.0");

trace("//\"\" >> \"-0.0\"");
assert_rshift("", "-0.0");

trace("//\"str\" >> \"-0.0\"");
assert_rshift("str", "-0.0");

trace("//\"true\" >> \"-0.0\"");
assert_rshift("true", "-0.0");

trace("//\"false\" >> \"-0.0\"");
assert_rshift("false", "-0.0");

trace("//0.0 >> \"-0.0\"");
assert_rshift(0.0, "-0.0");

trace("//NaN >> \"-0.0\"");
assert_rshift(NaN, "-0.0");

trace("//-0.0 >> \"-0.0\"");
assert_rshift(-0.0, "-0.0");

trace("//Infinity >> \"-0.0\"");
assert_rshift(Infinity, "-0.0");

trace("//1.0 >> \"-0.0\"");
assert_rshift(1.0, "-0.0");

trace("//-1.0 >> \"-0.0\"");
assert_rshift(-1.0, "-0.0");

trace("//0xFF1306 >> \"-0.0\"");
assert_rshift(0xFF1306, "-0.0");

trace("//new Object() >> \"-0.0\"");
assert_rshift({}, "-0.0");

trace("//\"0.0\" >> \"-0.0\"");
assert_rshift("0.0", "-0.0");

trace("//\"NaN\" >> \"-0.0\"");
assert_rshift("NaN", "-0.0");

trace("//\"-0.0\" >> \"-0.0\"");
assert_rshift("-0.0", "-0.0");

trace("//\"Infinity\" >> \"-0.0\"");
assert_rshift("Infinity", "-0.0");

trace("//\"1.0\" >> \"-0.0\"");
assert_rshift("1.0", "-0.0");

trace("//\"-1.0\" >> \"-0.0\"");
assert_rshift("-1.0", "-0.0");

trace("//\"0xFF1306\" >> \"-0.0\"");
assert_rshift("0xFF1306", "-0.0");

trace("//true >> \"Infinity\"");
assert_rshift(true, "Infinity");

trace("//false >> \"Infinity\"");
assert_rshift(false, "Infinity");

trace("//null >> \"Infinity\"");
assert_rshift(null, "Infinity");

trace("//undefined >> \"Infinity\"");
assert_rshift(undefined, "Infinity");

trace("//\"\" >> \"Infinity\"");
assert_rshift("", "Infinity");

trace("//\"str\" >> \"Infinity\"");
assert_rshift("str", "Infinity");

trace("//\"true\" >> \"Infinity\"");
assert_rshift("true", "Infinity");

trace("//\"false\" >> \"Infinity\"");
assert_rshift("false", "Infinity");

trace("//0.0 >> \"Infinity\"");
assert_rshift(0.0, "Infinity");

trace("//NaN >> \"Infinity\"");
assert_rshift(NaN, "Infinity");

trace("//-0.0 >> \"Infinity\"");
assert_rshift(-0.0, "Infinity");

trace("//Infinity >> \"Infinity\"");
assert_rshift(Infinity, "Infinity");

trace("//1.0 >> \"Infinity\"");
assert_rshift(1.0, "Infinity");

trace("//-1.0 >> \"Infinity\"");
assert_rshift(-1.0, "Infinity");

trace("//0xFF1306 >> \"Infinity\"");
assert_rshift(0xFF1306, "Infinity");

trace("//new Object() >> \"Infinity\"");
assert_rshift({}, "Infinity");

trace("//\"0.0\" >> \"Infinity\"");
assert_rshift("0.0", "Infinity");

trace("//\"NaN\" >> \"Infinity\"");
assert_rshift("NaN", "Infinity");

trace("//\"-0.0\" >> \"Infinity\"");
assert_rshift("-0.0", "Infinity");

trace("//\"Infinity\" >> \"Infinity\"");
assert_rshift("Infinity", "Infinity");

trace("//\"1.0\" >> \"Infinity\"");
assert_rshift("1.0", "Infinity");

trace("//\"-1.0\" >> \"Infinity\"");
assert_rshift("-1.0", "Infinity");

trace("//\"0xFF1306\" >> \"Infinity\"");
assert_rshift("0xFF1306", "Infinity");

trace("//true >> \"1.0\"");
assert_rshift(true, "1.0");

trace("//false >> \"1.0\"");
assert_rshift(false, "1.0");

trace("//null >> \"1.0\"");
assert_rshift(null, "1.0");

trace("//undefined >> \"1.0\"");
assert_rshift(undefined, "1.0");

trace("//\"\" >> \"1.0\"");
assert_rshift("", "1.0");

trace("//\"str\" >> \"1.0\"");
assert_rshift("str", "1.0");

trace("//\"true\" >> \"1.0\"");
assert_rshift("true", "1.0");

trace("//\"false\" >> \"1.0\"");
assert_rshift("false", "1.0");

trace("//0.0 >> \"1.0\"");
assert_rshift(0.0, "1.0");

trace("//NaN >> \"1.0\"");
assert_rshift(NaN, "1.0");

trace("//-0.0 >> \"1.0\"");
assert_rshift(-0.0, "1.0");

trace("//Infinity >> \"1.0\"");
assert_rshift(Infinity, "1.0");

trace("//1.0 >> \"1.0\"");
assert_rshift(1.0, "1.0");

trace("//-1.0 >> \"1.0\"");
assert_rshift(-1.0, "1.0");

trace("//0xFF1306 >> \"1.0\"");
assert_rshift(0xFF1306, "1.0");

trace("//new Object() >> \"1.0\"");
assert_rshift({}, "1.0");

trace("//\"0.0\" >> \"1.0\"");
assert_rshift("0.0", "1.0");

trace("//\"NaN\" >> \"1.0\"");
assert_rshift("NaN", "1.0");

trace("//\"-0.0\" >> \"1.0\"");
assert_rshift("-0.0", "1.0");

trace("//\"Infinity\" >> \"1.0\"");
assert_rshift("Infinity", "1.0");

trace("//\"1.0\" >> \"1.0\"");
assert_rshift("1.0", "1.0");

trace("//\"-1.0\" >> \"1.0\"");
assert_rshift("-1.0", "1.0");

trace("//\"0xFF1306\" >> \"1.0\"");
assert_rshift("0xFF1306", "1.0");

trace("//true >> \"-1.0\"");
assert_rshift(true, "-1.0");

trace("//false >> \"-1.0\"");
assert_rshift(false, "-1.0");

trace("//null >> \"-1.0\"");
assert_rshift(null, "-1.0");

trace("//undefined >> \"-1.0\"");
assert_rshift(undefined, "-1.0");

trace("//\"\" >> \"-1.0\"");
assert_rshift("", "-1.0");

trace("//\"str\" >> \"-1.0\"");
assert_rshift("str", "-1.0");

trace("//\"true\" >> \"-1.0\"");
assert_rshift("true", "-1.0");

trace("//\"false\" >> \"-1.0\"");
assert_rshift("false", "-1.0");

trace("//0.0 >> \"-1.0\"");
assert_rshift(0.0, "-1.0");

trace("//NaN >> \"-1.0\"");
assert_rshift(NaN, "-1.0");

trace("//-0.0 >> \"-1.0\"");
assert_rshift(-0.0, "-1.0");

trace("//Infinity >> \"-1.0\"");
assert_rshift(Infinity, "-1.0");

trace("//1.0 >> \"-1.0\"");
assert_rshift(1.0, "-1.0");

trace("//-1.0 >> \"-1.0\"");
assert_rshift(-1.0, "-1.0");

trace("//0xFF1306 >> \"-1.0\"");
assert_rshift(0xFF1306, "-1.0");

trace("//new Object() >> \"-1.0\"");
assert_rshift({}, "-1.0");

trace("//\"0.0\" >> \"-1.0\"");
assert_rshift("0.0", "-1.0");

trace("//\"NaN\" >> \"-1.0\"");
assert_rshift("NaN", "-1.0");

trace("//\"-0.0\" >> \"-1.0\"");
assert_rshift("-0.0", "-1.0");

trace("//\"Infinity\" >> \"-1.0\"");
assert_rshift("Infinity", "-1.0");

trace("//\"1.0\" >> \"-1.0\"");
assert_rshift("1.0", "-1.0");

trace("//\"-1.0\" >> \"-1.0\"");
assert_rshift("-1.0", "-1.0");

trace("//\"0xFF1306\" >> \"-1.0\"");
assert_rshift("0xFF1306", "-1.0");

trace("//true >> \"0xFF1306\"");
assert_rshift(true, "0xFF1306");

trace("//false >> \"0xFF1306\"");
assert_rshift(false, "0xFF1306");

trace("//null >> \"0xFF1306\"");
assert_rshift(null, "0xFF1306");

trace("//undefined >> \"0xFF1306\"");
assert_rshift(undefined, "0xFF1306");

trace("//\"\" >> \"0xFF1306\"");
assert_rshift("", "0xFF1306");

trace("//\"str\" >> \"0xFF1306\"");
assert_rshift("str", "0xFF1306");

trace("//\"true\" >> \"0xFF1306\"");
assert_rshift("true", "0xFF1306");

trace("//\"false\" >> \"0xFF1306\"");
assert_rshift("false", "0xFF1306");

trace("//0.0 >> \"0xFF1306\"");
assert_rshift(0.0, "0xFF1306");

trace("//NaN >> \"0xFF1306\"");
assert_rshift(NaN, "0xFF1306");

trace("//-0.0 >> \"0xFF1306\"");
assert_rshift(-0.0, "0xFF1306");

trace("//Infinity >> \"0xFF1306\"");
assert_rshift(Infinity, "0xFF1306");

trace("//1.0 >> \"0xFF1306\"");
assert_rshift(1.0, "0xFF1306");

trace("//-1.0 >> \"0xFF1306\"");
assert_rshift(-1.0, "0xFF1306");

trace("//0xFF1306 >> \"0xFF1306\"");
assert_rshift(0xFF1306, "0xFF1306");

trace("//new Object() >> \"0xFF1306\"");
assert_rshift({}, "0xFF1306");

trace("//\"0.0\" >> \"0xFF1306\"");
assert_rshift("0.0", "0xFF1306");

trace("//\"NaN\" >> \"0xFF1306\"");
assert_rshift("NaN", "0xFF1306");

trace("//\"-0.0\" >> \"0xFF1306\"");
assert_rshift("-0.0", "0xFF1306");

trace("//\"Infinity\" >> \"0xFF1306\"");
assert_rshift("Infinity", "0xFF1306");

trace("//\"1.0\" >> \"0xFF1306\"");
assert_rshift("1.0", "0xFF1306");

trace("//\"-1.0\" >> \"0xFF1306\"");
assert_rshift("-1.0", "0xFF1306");

trace("//\"0xFF1306\" >> \"0xFF1306\"");
assert_rshift("0xFF1306", "0xFF1306");