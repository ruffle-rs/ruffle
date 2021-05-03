package {
	public class Test {
	}
}

function assert_lshift(val1, val2) {
	trace(val1 << val2);
}

trace("//true << true");
assert_lshift(true, true);

trace("//false << true");
assert_lshift(false, true);

trace("//null << true");
assert_lshift(null, true);

trace("//undefined << true");
assert_lshift(undefined, true);

trace("//\"\" << true");
assert_lshift("", true);

trace("//\"str\" << true");
assert_lshift("str", true);

trace("//\"true\" << true");
assert_lshift("true", true);

trace("//\"false\" << true");
assert_lshift("false", true);

trace("//0.0 << true");
assert_lshift(0.0, true);

trace("//NaN << true");
assert_lshift(NaN, true);

trace("//-0.0 << true");
assert_lshift(-0.0, true);

trace("//Infinity << true");
assert_lshift(Infinity, true);

trace("//1.0 << true");
assert_lshift(1.0, true);

trace("//-1.0 << true");
assert_lshift(-1.0, true);

trace("//0xFF1306 << true");
assert_lshift(0xFF1306, true);

trace("//new Object() << true");
assert_lshift({}, true);

trace("//\"0.0\" << true");
assert_lshift("0.0", true);

trace("//\"NaN\" << true");
assert_lshift("NaN", true);

trace("//\"-0.0\" << true");
assert_lshift("-0.0", true);

trace("//\"Infinity\" << true");
assert_lshift("Infinity", true);

trace("//\"1.0\" << true");
assert_lshift("1.0", true);

trace("//\"-1.0\" << true");
assert_lshift("-1.0", true);

trace("//\"0xFF1306\" << true");
assert_lshift("0xFF1306", true);

trace("//true << false");
assert_lshift(true, false);

trace("//false << false");
assert_lshift(false, false);

trace("//null << false");
assert_lshift(null, false);

trace("//undefined << false");
assert_lshift(undefined, false);

trace("//\"\" << false");
assert_lshift("", false);

trace("//\"str\" << false");
assert_lshift("str", false);

trace("//\"true\" << false");
assert_lshift("true", false);

trace("//\"false\" << false");
assert_lshift("false", false);

trace("//0.0 << false");
assert_lshift(0.0, false);

trace("//NaN << false");
assert_lshift(NaN, false);

trace("//-0.0 << false");
assert_lshift(-0.0, false);

trace("//Infinity << false");
assert_lshift(Infinity, false);

trace("//1.0 << false");
assert_lshift(1.0, false);

trace("//-1.0 << false");
assert_lshift(-1.0, false);

trace("//0xFF1306 << false");
assert_lshift(0xFF1306, false);

trace("//new Object() << false");
assert_lshift({}, false);

trace("//\"0.0\" << false");
assert_lshift("0.0", false);

trace("//\"NaN\" << false");
assert_lshift("NaN", false);

trace("//\"-0.0\" << false");
assert_lshift("-0.0", false);

trace("//\"Infinity\" << false");
assert_lshift("Infinity", false);

trace("//\"1.0\" << false");
assert_lshift("1.0", false);

trace("//\"-1.0\" << false");
assert_lshift("-1.0", false);

trace("//\"0xFF1306\" << false");
assert_lshift("0xFF1306", false);
trace("//true << null");
assert_lshift(true, null);

trace("//false << null");
assert_lshift(false, null);

trace("//null << null");
assert_lshift(null, null);

trace("//undefined << null");
assert_lshift(undefined, null);

trace("//\"\" << null");
assert_lshift("", null);

trace("//\"str\" << null");
assert_lshift("str", null);

trace("//\"true\" << null");
assert_lshift("true", null);

trace("//\"false\" << null");
assert_lshift("false", null);

trace("//0.0 << null");
assert_lshift(0.0, null);

trace("//NaN << null");
assert_lshift(NaN, null);

trace("//-0.0 << null");
assert_lshift(-0.0, null);

trace("//Infinity << null");
assert_lshift(Infinity, null);

trace("//1.0 << null");
assert_lshift(1.0, null);

trace("//-1.0 << null");
assert_lshift(-1.0, null);

trace("//0xFF1306 << null");
assert_lshift(0xFF1306, null);

trace("//new Object() << null");
assert_lshift({}, null);

trace("//\"0.0\" << null");
assert_lshift("0.0", null);

trace("//\"NaN\" << null");
assert_lshift("NaN", null);

trace("//\"-0.0\" << null");
assert_lshift("-0.0", null);

trace("//\"Infinity\" << null");
assert_lshift("Infinity", null);

trace("//\"1.0\" << null");
assert_lshift("1.0", null);

trace("//\"-1.0\" << null");
assert_lshift("-1.0", null);

trace("//\"0xFF1306\" << null");
assert_lshift("0xFF1306", null);

trace("//true << undefined");
assert_lshift(true, undefined);

trace("//false << undefined");
assert_lshift(false, undefined);

trace("//null << undefined");
assert_lshift(null, undefined);

trace("//undefined << undefined");
assert_lshift(undefined, undefined);

trace("//\"\" << undefined");
assert_lshift("", undefined);

trace("//\"str\" << undefined");
assert_lshift("str", undefined);

trace("//\"true\" << undefined");
assert_lshift("true", undefined);

trace("//\"false\" << undefined");
assert_lshift("false", undefined);

trace("//0.0 << undefined");
assert_lshift(0.0, undefined);

trace("//NaN << undefined");
assert_lshift(NaN, undefined);

trace("//-0.0 << undefined");
assert_lshift(-0.0, undefined);

trace("//Infinity << undefined");
assert_lshift(Infinity, undefined);

trace("//1.0 << undefined");
assert_lshift(1.0, undefined);

trace("//-1.0 << undefined");
assert_lshift(-1.0, undefined);

trace("//0xFF1306 << undefined");
assert_lshift(0xFF1306, undefined);

trace("//new Object() << undefined");
assert_lshift({}, undefined);

trace("//\"0.0\" << undefined");
assert_lshift("0.0", undefined);

trace("//\"NaN\" << undefined");
assert_lshift("NaN", undefined);

trace("//\"-0.0\" << undefined");
assert_lshift("-0.0", undefined);

trace("//\"Infinity\" << undefined");
assert_lshift("Infinity", undefined);

trace("//\"1.0\" << undefined");
assert_lshift("1.0", undefined);

trace("//\"-1.0\" << undefined");
assert_lshift("-1.0", undefined);

trace("//\"0xFF1306\" << undefined");
assert_lshift("0xFF1306", undefined);

trace("//true << \"\"");
assert_lshift(true, "");

trace("//false << \"\"");
assert_lshift(false, "");

trace("//null << \"\"");
assert_lshift(null, "");

trace("//undefined << \"\"");
assert_lshift(undefined, "");

trace("//\"\" << \"\"");
assert_lshift("", "");

trace("//\"str\" << \"\"");
assert_lshift("str", "");

trace("//\"true\" << \"\"");
assert_lshift("true", "");

trace("//\"false\" << \"\"");
assert_lshift("false", "");

trace("//0.0 << \"\"");
assert_lshift(0.0, "");

trace("//NaN << \"\"");
assert_lshift(NaN, "");

trace("//-0.0 << \"\"");
assert_lshift(-0.0, "");

trace("//Infinity << \"\"");
assert_lshift(Infinity, "");

trace("//1.0 << \"\"");
assert_lshift(1.0, "");

trace("//-1.0 << \"\"");
assert_lshift(-1.0, "");

trace("//0xFF1306 << \"\"");
assert_lshift(0xFF1306, "");

trace("//new Object() << \"\"");
assert_lshift({}, "");

trace("//\"0.0\" << \"\"");
assert_lshift("0.0", "");

trace("//\"NaN\" << \"\"");
assert_lshift("NaN", "");

trace("//\"-0.0\" << \"\"");
assert_lshift("-0.0", "");

trace("//\"Infinity\" << \"\"");
assert_lshift("Infinity", "");

trace("//\"1.0\" << \"\"");
assert_lshift("1.0", "");

trace("//\"-1.0\" << \"\"");
assert_lshift("-1.0", "");

trace("//\"0xFF1306\" << \"\"");
assert_lshift("0xFF1306", "");

trace("//true << \"str\"");
assert_lshift(true, "str");

trace("//false << \"str\"");
assert_lshift(false, "str");

trace("//null << \"str\"");
assert_lshift(null, "str");

trace("//undefined << \"str\"");
assert_lshift(undefined, "str");

trace("//\"\" << \"str\"");
assert_lshift("", "str");

trace("//\"str\" << \"str\"");
assert_lshift("str", "str");

trace("//\"true\" << \"str\"");
assert_lshift("true", "str");

trace("//\"false\" << \"str\"");
assert_lshift("false", "str");

trace("//0.0 << \"str\"");
assert_lshift(0.0, "str");

trace("//NaN << \"str\"");
assert_lshift(NaN, "str");

trace("//-0.0 << \"str\"");
assert_lshift(-0.0, "str");

trace("//Infinity << \"str\"");
assert_lshift(Infinity, "str");

trace("//1.0 << \"str\"");
assert_lshift(1.0, "str");

trace("//-1.0 << \"str\"");
assert_lshift(-1.0, "str");

trace("//0xFF1306 << \"str\"");
assert_lshift(0xFF1306, "str");

trace("//new Object() << \"str\"");
assert_lshift({}, "str");

trace("//\"0.0\" << \"str\"");
assert_lshift("0.0", "str");

trace("//\"NaN\" << \"str\"");
assert_lshift("NaN", "str");

trace("//\"-0.0\" << \"str\"");
assert_lshift("-0.0", "str");

trace("//\"Infinity\" << \"str\"");
assert_lshift("Infinity", "str");

trace("//\"1.0\" << \"str\"");
assert_lshift("1.0", "str");

trace("//\"-1.0\" << \"str\"");
assert_lshift("-1.0", "str");

trace("//\"0xFF1306\" << \"str\"");
assert_lshift("0xFF1306", "str");

trace("//true << \"true\"");
assert_lshift(true, "true");

trace("//false << \"true\"");
assert_lshift(false, "true");

trace("//null << \"true\"");
assert_lshift(null, "true");

trace("//undefined << \"true\"");
assert_lshift(undefined, "true");

trace("//\"\" << \"true\"");
assert_lshift("", "true");

trace("//\"str\" << \"true\"");
assert_lshift("str", "true");

trace("//\"true\" << \"true\"");
assert_lshift("true", "true");

trace("//\"false\" << \"true\"");
assert_lshift("false", "true");

trace("//0.0 << \"true\"");
assert_lshift(0.0, "true");

trace("//NaN << \"true\"");
assert_lshift(NaN, "true");

trace("//-0.0 << \"true\"");
assert_lshift(-0.0, "true");

trace("//Infinity << \"true\"");
assert_lshift(Infinity, "true");

trace("//1.0 << \"true\"");
assert_lshift(1.0, "true");

trace("//-1.0 << \"true\"");
assert_lshift(-1.0, "true");

trace("//0xFF1306 << \"true\"");
assert_lshift(0xFF1306, "true");

trace("//new Object() << \"true\"");
assert_lshift({}, "true");

trace("//\"0.0\" << \"true\"");
assert_lshift("0.0", "true");

trace("//\"NaN\" << \"true\"");
assert_lshift("NaN", "true");

trace("//\"-0.0\" << \"true\"");
assert_lshift("-0.0", "true");

trace("//\"Infinity\" << \"true\"");
assert_lshift("Infinity", "true");

trace("//\"1.0\" << \"true\"");
assert_lshift("1.0", "true");

trace("//\"-1.0\" << \"true\"");
assert_lshift("-1.0", "true");

trace("//\"0xFF1306\" << \"true\"");
assert_lshift("0xFF1306", "true");

trace("//true << \"false\"");
assert_lshift(true, "false");

trace("//false << \"false\"");
assert_lshift(false, "false");

trace("//null << \"false\"");
assert_lshift(null, "false");

trace("//undefined << \"false\"");
assert_lshift(undefined, "false");

trace("//\"\" << \"false\"");
assert_lshift("", "false");

trace("//\"str\" << \"false\"");
assert_lshift("str", "false");

trace("//\"true\" << \"false\"");
assert_lshift("true", "false");

trace("//\"false\" << \"false\"");
assert_lshift("false", "false");

trace("//0.0 << \"false\"");
assert_lshift(0.0, "false");

trace("//NaN << \"false\"");
assert_lshift(NaN, "false");

trace("//-0.0 << \"false\"");
assert_lshift(-0.0, "false");

trace("//Infinity << \"false\"");
assert_lshift(Infinity, "false");

trace("//1.0 << \"false\"");
assert_lshift(1.0, "false");

trace("//-1.0 << \"false\"");
assert_lshift(-1.0, "false");

trace("//0xFF1306 << \"false\"");
assert_lshift(0xFF1306, "false");

trace("//new Object() << \"false\"");
assert_lshift({}, "false");

trace("//\"0.0\" << \"false\"");
assert_lshift("0.0", "false");

trace("//\"NaN\" << \"false\"");
assert_lshift("NaN", "false");

trace("//\"-0.0\" << \"false\"");
assert_lshift("-0.0", "false");

trace("//\"Infinity\" << \"false\"");
assert_lshift("Infinity", "false");

trace("//\"1.0\" << \"false\"");
assert_lshift("1.0", "false");

trace("//\"-1.0\" << \"false\"");
assert_lshift("-1.0", "false");

trace("//\"0xFF1306\" << \"false\"");
assert_lshift("0xFF1306", "false");

trace("//true << 0.0");
assert_lshift(true, 0.0);

trace("//false << 0.0");
assert_lshift(false, 0.0);

trace("//null << 0.0");
assert_lshift(null, 0.0);

trace("//undefined << 0.0");
assert_lshift(undefined, 0.0);

trace("//\"\" << 0.0");
assert_lshift("", 0.0);

trace("//\"str\" << 0.0");
assert_lshift("str", 0.0);

trace("//\"true\" << 0.0");
assert_lshift("true", 0.0);

trace("//\"false\" << 0.0");
assert_lshift("false", 0.0);

trace("//0.0 << 0.0");
assert_lshift(0.0, 0.0);

trace("//NaN << 0.0");
assert_lshift(NaN, 0.0);

trace("//-0.0 << 0.0");
assert_lshift(-0.0, 0.0);

trace("//Infinity << 0.0");
assert_lshift(Infinity, 0.0);

trace("//1.0 << 0.0");
assert_lshift(1.0, 0.0);

trace("//-1.0 << 0.0");
assert_lshift(-1.0, 0.0);

trace("//0xFF1306 << 0.0");
assert_lshift(0xFF1306, 0.0);

trace("//new Object() << 0.0");
assert_lshift({}, 0.0);

trace("//\"0.0\" << 0.0");
assert_lshift("0.0", 0.0);

trace("//\"NaN\" << 0.0");
assert_lshift("NaN", 0.0);

trace("//\"-0.0\" << 0.0");
assert_lshift("-0.0", 0.0);

trace("//\"Infinity\" << 0.0");
assert_lshift("Infinity", 0.0);

trace("//\"1.0\" << 0.0");
assert_lshift("1.0", 0.0);

trace("//\"-1.0\" << 0.0");
assert_lshift("-1.0", 0.0);

trace("//\"0xFF1306\" << 0.0");
assert_lshift("0xFF1306", 0.0);

trace("//true << NaN");
assert_lshift(true, NaN);

trace("//false << NaN");
assert_lshift(false, NaN);

trace("//null << NaN");
assert_lshift(null, NaN);

trace("//undefined << NaN");
assert_lshift(undefined, NaN);

trace("//\"\" << NaN");
assert_lshift("", NaN);

trace("//\"str\" << NaN");
assert_lshift("str", NaN);

trace("//\"true\" << NaN");
assert_lshift("true", NaN);

trace("//\"false\" << NaN");
assert_lshift("false", NaN);

trace("//0.0 << NaN");
assert_lshift(0.0, NaN);

trace("//NaN << NaN");
assert_lshift(NaN, NaN);

trace("//-0.0 << NaN");
assert_lshift(-0.0, NaN);

trace("//Infinity << NaN");
assert_lshift(Infinity, NaN);

trace("//1.0 << NaN");
assert_lshift(1.0, NaN);

trace("//-1.0 << NaN");
assert_lshift(-1.0, NaN);

trace("//0xFF1306 << NaN");
assert_lshift(0xFF1306, NaN);

trace("//new Object() << NaN");
assert_lshift({}, NaN);

trace("//\"0.0\" << NaN");
assert_lshift("0.0", NaN);

trace("//\"NaN\" << NaN");
assert_lshift("NaN", NaN);

trace("//\"-0.0\" << NaN");
assert_lshift("-0.0", NaN);

trace("//\"Infinity\" << NaN");
assert_lshift("Infinity", NaN);

trace("//\"1.0\" << NaN");
assert_lshift("1.0", NaN);

trace("//\"-1.0\" << NaN");
assert_lshift("-1.0", NaN);

trace("//\"0xFF1306\" << NaN");
assert_lshift("0xFF1306", NaN);

trace("//true << -0.0");
assert_lshift(true, -0.0);

trace("//false << -0.0");
assert_lshift(false, -0.0);

trace("//null << -0.0");
assert_lshift(null, -0.0);

trace("//undefined << -0.0");
assert_lshift(undefined, -0.0);

trace("//\"\" << -0.0");
assert_lshift("", -0.0);

trace("//\"str\" << -0.0");
assert_lshift("str", -0.0);

trace("//\"true\" << -0.0");
assert_lshift("true", -0.0);

trace("//\"false\" << -0.0");
assert_lshift("false", -0.0);

trace("//0.0 << -0.0");
assert_lshift(0.0, -0.0);

trace("//NaN << -0.0");
assert_lshift(NaN, -0.0);

trace("//-0.0 << -0.0");
assert_lshift(-0.0, -0.0);

trace("//Infinity << -0.0");
assert_lshift(Infinity, -0.0);

trace("//1.0 << -0.0");
assert_lshift(1.0, -0.0);

trace("//-1.0 << -0.0");
assert_lshift(-1.0, -0.0);

trace("//0xFF1306 << -0.0");
assert_lshift(0xFF1306, -0.0);

trace("//new Object() << -0.0");
assert_lshift({}, -0.0);

trace("//\"0.0\" << -0.0");
assert_lshift("0.0", -0.0);

trace("//\"NaN\" << -0.0");
assert_lshift("NaN", -0.0);

trace("//\"-0.0\" << -0.0");
assert_lshift("-0.0", -0.0);

trace("//\"Infinity\" << -0.0");
assert_lshift("Infinity", -0.0);

trace("//\"1.0\" << -0.0");
assert_lshift("1.0", -0.0);

trace("//\"-1.0\" << -0.0");
assert_lshift("-1.0", -0.0);

trace("//\"0xFF1306\" << -0.0");
assert_lshift("0xFF1306", -0.0);

trace("//true << Infinity");
assert_lshift(true, Infinity);

trace("//false << Infinity");
assert_lshift(false, Infinity);

trace("//null << Infinity");
assert_lshift(null, Infinity);

trace("//undefined << Infinity");
assert_lshift(undefined, Infinity);

trace("//\"\" << Infinity");
assert_lshift("", Infinity);

trace("//\"str\" << Infinity");
assert_lshift("str", Infinity);

trace("//\"true\" << Infinity");
assert_lshift("true", Infinity);

trace("//\"false\" << Infinity");
assert_lshift("false", Infinity);

trace("//0.0 << Infinity");
assert_lshift(0.0, Infinity);

trace("//NaN << Infinity");
assert_lshift(NaN, Infinity);

trace("//-0.0 << Infinity");
assert_lshift(-0.0, Infinity);

trace("//Infinity << Infinity");
assert_lshift(Infinity, Infinity);

trace("//1.0 << Infinity");
assert_lshift(1.0, Infinity);

trace("//-1.0 << Infinity");
assert_lshift(-1.0, Infinity);

trace("//0xFF1306 << Infinity");
assert_lshift(0xFF1306, Infinity);

trace("//new Object() << Infinity");
assert_lshift({}, Infinity);

trace("//\"0.0\" << Infinity");
assert_lshift("0.0", Infinity);

trace("//\"NaN\" << Infinity");
assert_lshift("NaN", Infinity);

trace("//\"-0.0\" << Infinity");
assert_lshift("-0.0", Infinity);

trace("//\"Infinity\" << Infinity");
assert_lshift("Infinity", Infinity);

trace("//\"1.0\" << Infinity");
assert_lshift("1.0", Infinity);

trace("//\"-1.0\" << Infinity");
assert_lshift("-1.0", Infinity);

trace("//\"0xFF1306\" << Infinity");
assert_lshift("0xFF1306", Infinity);

trace("//true << 1.0");
assert_lshift(true, 1.0);

trace("//false << 1.0");
assert_lshift(false, 1.0);

trace("//null << 1.0");
assert_lshift(null, 1.0);

trace("//undefined << 1.0");
assert_lshift(undefined, 1.0);

trace("//\"\" << 1.0");
assert_lshift("", 1.0);

trace("//\"str\" << 1.0");
assert_lshift("str", 1.0);

trace("//\"true\" << 1.0");
assert_lshift("true", 1.0);

trace("//\"false\" << 1.0");
assert_lshift("false", 1.0);

trace("//0.0 << 1.0");
assert_lshift(0.0, 1.0);

trace("//NaN << 1.0");
assert_lshift(NaN, 1.0);

trace("//-0.0 << 1.0");
assert_lshift(-0.0, 1.0);

trace("//Infinity << 1.0");
assert_lshift(Infinity, 1.0);

trace("//1.0 << 1.0");
assert_lshift(1.0, 1.0);

trace("//-1.0 << 1.0");
assert_lshift(-1.0, 1.0);

trace("//0xFF1306 << 1.0");
assert_lshift(0xFF1306, 1.0);

trace("//new Object() << 1.0");
assert_lshift({}, 1.0);

trace("//\"0.0\" << 1.0");
assert_lshift("0.0", 1.0);

trace("//\"NaN\" << 1.0");
assert_lshift("NaN", 1.0);

trace("//\"-0.0\" << 1.0");
assert_lshift("-0.0", 1.0);

trace("//\"Infinity\" << 1.0");
assert_lshift("Infinity", 1.0);

trace("//\"1.0\" << 1.0");
assert_lshift("1.0", 1.0);

trace("//\"-1.0\" << 1.0");
assert_lshift("-1.0", 1.0);

trace("//\"0xFF1306\" << 1.0");
assert_lshift("0xFF1306", 1.0);

trace("//true << -1.0");
assert_lshift(true, -1.0);

trace("//false << -1.0");
assert_lshift(false, -1.0);

trace("//null << -1.0");
assert_lshift(null, -1.0);

trace("//undefined << -1.0");
assert_lshift(undefined, -1.0);

trace("//\"\" << -1.0");
assert_lshift("", -1.0);

trace("//\"str\" << -1.0");
assert_lshift("str", -1.0);

trace("//\"true\" << -1.0");
assert_lshift("true", -1.0);

trace("//\"false\" << -1.0");
assert_lshift("false", -1.0);

trace("//0.0 << -1.0");
assert_lshift(0.0, -1.0);

trace("//NaN << -1.0");
assert_lshift(NaN, -1.0);

trace("//-0.0 << -1.0");
assert_lshift(-0.0, -1.0);

trace("//Infinity << -1.0");
assert_lshift(Infinity, -1.0);

trace("//1.0 << -1.0");
assert_lshift(1.0, -1.0);

trace("//-1.0 << -1.0");
assert_lshift(-1.0, -1.0);

trace("//0xFF1306 << -1.0");
assert_lshift(0xFF1306, -1.0);

trace("//new Object() << -1.0");
assert_lshift({}, -1.0);

trace("//\"0.0\" << -1.0");
assert_lshift("0.0", -1.0);

trace("//\"NaN\" << -1.0");
assert_lshift("NaN", -1.0);

trace("//\"-0.0\" << -1.0");
assert_lshift("-0.0", -1.0);

trace("//\"Infinity\" << -1.0");
assert_lshift("Infinity", -1.0);

trace("//\"1.0\" << -1.0");
assert_lshift("1.0", -1.0);

trace("//\"-1.0\" << -1.0");
assert_lshift("-1.0", -1.0);

trace("//\"0xFF1306\" << -1.0");
assert_lshift("0xFF1306", -1.0);

trace("//true << 0xFF1306");
assert_lshift(true, 0xFF1306);

trace("//false << 0xFF1306");
assert_lshift(false, 0xFF1306);

trace("//null << 0xFF1306");
assert_lshift(null, 0xFF1306);

trace("//undefined << 0xFF1306");
assert_lshift(undefined, 0xFF1306);

trace("//\"\" << 0xFF1306");
assert_lshift("", 0xFF1306);

trace("//\"str\" << 0xFF1306");
assert_lshift("str", 0xFF1306);

trace("//\"true\" << 0xFF1306");
assert_lshift("true", 0xFF1306);

trace("//\"false\" << 0xFF1306");
assert_lshift("false", 0xFF1306);

trace("//0.0 << 0xFF1306");
assert_lshift(0.0, 0xFF1306);

trace("//NaN << 0xFF1306");
assert_lshift(NaN, 0xFF1306);

trace("//-0.0 << 0xFF1306");
assert_lshift(-0.0, 0xFF1306);

trace("//Infinity << 0xFF1306");
assert_lshift(Infinity, 0xFF1306);

trace("//1.0 << 0xFF1306");
assert_lshift(1.0, 0xFF1306);

trace("//-1.0 << 0xFF1306");
assert_lshift(-1.0, 0xFF1306);

trace("//0xFF1306 << 0xFF1306");
assert_lshift(0xFF1306, 0xFF1306);

trace("//new Object() << 0xFF1306");
assert_lshift({}, 0xFF1306);

trace("//\"0.0\" << 0xFF1306");
assert_lshift("0.0", 0xFF1306);

trace("//\"NaN\" << 0xFF1306");
assert_lshift("NaN", 0xFF1306);

trace("//\"-0.0\" << 0xFF1306");
assert_lshift("-0.0", 0xFF1306);

trace("//\"Infinity\" << 0xFF1306");
assert_lshift("Infinity", 0xFF1306);

trace("//\"1.0\" << 0xFF1306");
assert_lshift("1.0", 0xFF1306);

trace("//\"-1.0\" << 0xFF1306");
assert_lshift("-1.0", 0xFF1306);

trace("//\"0xFF1306\" << 0xFF1306");
assert_lshift("0xFF1306", 0xFF1306);

trace("//true << new Object()");
assert_lshift(true, {});

trace("//false << new Object()");
assert_lshift(false, {});

trace("//null << new Object()");
assert_lshift(null, {});

trace("//undefined << new Object()");
assert_lshift(undefined, {});

trace("//\"\" << new Object()");
assert_lshift("", {});

trace("//\"str\" << new Object()");
assert_lshift("str", {});

trace("//\"true\" << new Object()");
assert_lshift("true", {});

trace("//\"false\" << new Object()");
assert_lshift("false", {});

trace("//0.0 << new Object()");
assert_lshift(0.0, {});

trace("//NaN << new Object()");
assert_lshift(NaN, {});

trace("//-0.0 << new Object()");
assert_lshift(-0.0, {});

trace("//Infinity << new Object()");
assert_lshift(Infinity, {});

trace("//1.0 << new Object()");
assert_lshift(1.0, {});

trace("//-1.0 << new Object()");
assert_lshift(-1.0, {});

trace("//0xFF1306 << new Object()");
assert_lshift(0xFF1306, {});

trace("//new Object() << new Object()");
assert_lshift({}, {});

trace("//\"0.0\" << new Object()");
assert_lshift("0.0", {});

trace("//\"NaN\" << new Object()");
assert_lshift("NaN", {});

trace("//\"-0.0\" << new Object()");
assert_lshift("-0.0", {});

trace("//\"Infinity\" << new Object()");
assert_lshift("Infinity", {});

trace("//\"1.0\" << new Object()");
assert_lshift("1.0", {});

trace("//\"-1.0\" << new Object()");
assert_lshift("-1.0", {});

trace("//\"0xFF1306\" << new Object()");
assert_lshift("0xFF1306", {});

trace("//true << \"0.0\"");
assert_lshift(true, "0.0");

trace("//false << \"0.0\"");
assert_lshift(false, "0.0");

trace("//null << \"0.0\"");
assert_lshift(null, "0.0");

trace("//undefined << \"0.0\"");
assert_lshift(undefined, "0.0");

trace("//\"\" << \"0.0\"");
assert_lshift("", "0.0");

trace("//\"str\" << \"0.0\"");
assert_lshift("str", "0.0");

trace("//\"true\" << \"0.0\"");
assert_lshift("true", "0.0");

trace("//\"false\" << \"0.0\"");
assert_lshift("false", "0.0");

trace("//0.0 << \"0.0\"");
assert_lshift(0.0, "0.0");

trace("//NaN << \"0.0\"");
assert_lshift(NaN, "0.0");

trace("//-0.0 << \"0.0\"");
assert_lshift(-0.0, "0.0");

trace("//Infinity << \"0.0\"");
assert_lshift(Infinity, "0.0");

trace("//1.0 << \"0.0\"");
assert_lshift(1.0, "0.0");

trace("//-1.0 << \"0.0\"");
assert_lshift(-1.0, "0.0");

trace("//0xFF1306 << \"0.0\"");
assert_lshift(0xFF1306, "0.0");

trace("//new Object() << \"0.0\"");
assert_lshift({}, "0.0");

trace("//\"0.0\" << \"0.0\"");
assert_lshift("0.0", "0.0");

trace("//\"NaN\" << \"0.0\"");
assert_lshift("NaN", "0.0");

trace("//\"-0.0\" << \"0.0\"");
assert_lshift("-0.0", "0.0");

trace("//\"Infinity\" << \"0.0\"");
assert_lshift("Infinity", "0.0");

trace("//\"1.0\" << \"0.0\"");
assert_lshift("1.0", "0.0");

trace("//\"-1.0\" << \"0.0\"");
assert_lshift("-1.0", "0.0");

trace("//\"0xFF1306\" << \"0.0\"");
assert_lshift("0xFF1306", "0.0");

trace("//true << \"NaN\"");
assert_lshift(true, "NaN");

trace("//false << \"NaN\"");
assert_lshift(false, "NaN");

trace("//null << \"NaN\"");
assert_lshift(null, "NaN");

trace("//undefined << \"NaN\"");
assert_lshift(undefined, "NaN");

trace("//\"\" << \"NaN\"");
assert_lshift("", "NaN");

trace("//\"str\" << \"NaN\"");
assert_lshift("str", "NaN");

trace("//\"true\" << \"NaN\"");
assert_lshift("true", "NaN");

trace("//\"false\" << \"NaN\"");
assert_lshift("false", "NaN");

trace("//0.0 << \"NaN\"");
assert_lshift(0.0, "NaN");

trace("//NaN << \"NaN\"");
assert_lshift(NaN, "NaN");

trace("//-0.0 << \"NaN\"");
assert_lshift(-0.0, "NaN");

trace("//Infinity << \"NaN\"");
assert_lshift(Infinity, "NaN");

trace("//1.0 << \"NaN\"");
assert_lshift(1.0, "NaN");

trace("//-1.0 << \"NaN\"");
assert_lshift(-1.0, "NaN");

trace("//0xFF1306 << \"NaN\"");
assert_lshift(0xFF1306, "NaN");

trace("//new Object() << \"NaN\"");
assert_lshift({}, "NaN");

trace("//\"0.0\" << \"NaN\"");
assert_lshift("0.0", "NaN");

trace("//\"NaN\" << \"NaN\"");
assert_lshift("NaN", "NaN");

trace("//\"-0.0\" << \"NaN\"");
assert_lshift("-0.0", "NaN");

trace("//\"Infinity\" << \"NaN\"");
assert_lshift("Infinity", "NaN");

trace("//\"1.0\" << \"NaN\"");
assert_lshift("1.0", "NaN");

trace("//\"-1.0\" << \"NaN\"");
assert_lshift("-1.0", "NaN");

trace("//\"0xFF1306\" << \"NaN\"");
assert_lshift("0xFF1306", "NaN");

trace("//true << \"-0.0\"");
assert_lshift(true, "-0.0");

trace("//false << \"-0.0\"");
assert_lshift(false, "-0.0");

trace("//null << \"-0.0\"");
assert_lshift(null, "-0.0");

trace("//undefined << \"-0.0\"");
assert_lshift(undefined, "-0.0");

trace("//\"\" << \"-0.0\"");
assert_lshift("", "-0.0");

trace("//\"str\" << \"-0.0\"");
assert_lshift("str", "-0.0");

trace("//\"true\" << \"-0.0\"");
assert_lshift("true", "-0.0");

trace("//\"false\" << \"-0.0\"");
assert_lshift("false", "-0.0");

trace("//0.0 << \"-0.0\"");
assert_lshift(0.0, "-0.0");

trace("//NaN << \"-0.0\"");
assert_lshift(NaN, "-0.0");

trace("//-0.0 << \"-0.0\"");
assert_lshift(-0.0, "-0.0");

trace("//Infinity << \"-0.0\"");
assert_lshift(Infinity, "-0.0");

trace("//1.0 << \"-0.0\"");
assert_lshift(1.0, "-0.0");

trace("//-1.0 << \"-0.0\"");
assert_lshift(-1.0, "-0.0");

trace("//0xFF1306 << \"-0.0\"");
assert_lshift(0xFF1306, "-0.0");

trace("//new Object() << \"-0.0\"");
assert_lshift({}, "-0.0");

trace("//\"0.0\" << \"-0.0\"");
assert_lshift("0.0", "-0.0");

trace("//\"NaN\" << \"-0.0\"");
assert_lshift("NaN", "-0.0");

trace("//\"-0.0\" << \"-0.0\"");
assert_lshift("-0.0", "-0.0");

trace("//\"Infinity\" << \"-0.0\"");
assert_lshift("Infinity", "-0.0");

trace("//\"1.0\" << \"-0.0\"");
assert_lshift("1.0", "-0.0");

trace("//\"-1.0\" << \"-0.0\"");
assert_lshift("-1.0", "-0.0");

trace("//\"0xFF1306\" << \"-0.0\"");
assert_lshift("0xFF1306", "-0.0");

trace("//true << \"Infinity\"");
assert_lshift(true, "Infinity");

trace("//false << \"Infinity\"");
assert_lshift(false, "Infinity");

trace("//null << \"Infinity\"");
assert_lshift(null, "Infinity");

trace("//undefined << \"Infinity\"");
assert_lshift(undefined, "Infinity");

trace("//\"\" << \"Infinity\"");
assert_lshift("", "Infinity");

trace("//\"str\" << \"Infinity\"");
assert_lshift("str", "Infinity");

trace("//\"true\" << \"Infinity\"");
assert_lshift("true", "Infinity");

trace("//\"false\" << \"Infinity\"");
assert_lshift("false", "Infinity");

trace("//0.0 << \"Infinity\"");
assert_lshift(0.0, "Infinity");

trace("//NaN << \"Infinity\"");
assert_lshift(NaN, "Infinity");

trace("//-0.0 << \"Infinity\"");
assert_lshift(-0.0, "Infinity");

trace("//Infinity << \"Infinity\"");
assert_lshift(Infinity, "Infinity");

trace("//1.0 << \"Infinity\"");
assert_lshift(1.0, "Infinity");

trace("//-1.0 << \"Infinity\"");
assert_lshift(-1.0, "Infinity");

trace("//0xFF1306 << \"Infinity\"");
assert_lshift(0xFF1306, "Infinity");

trace("//new Object() << \"Infinity\"");
assert_lshift({}, "Infinity");

trace("//\"0.0\" << \"Infinity\"");
assert_lshift("0.0", "Infinity");

trace("//\"NaN\" << \"Infinity\"");
assert_lshift("NaN", "Infinity");

trace("//\"-0.0\" << \"Infinity\"");
assert_lshift("-0.0", "Infinity");

trace("//\"Infinity\" << \"Infinity\"");
assert_lshift("Infinity", "Infinity");

trace("//\"1.0\" << \"Infinity\"");
assert_lshift("1.0", "Infinity");

trace("//\"-1.0\" << \"Infinity\"");
assert_lshift("-1.0", "Infinity");

trace("//\"0xFF1306\" << \"Infinity\"");
assert_lshift("0xFF1306", "Infinity");

trace("//true << \"1.0\"");
assert_lshift(true, "1.0");

trace("//false << \"1.0\"");
assert_lshift(false, "1.0");

trace("//null << \"1.0\"");
assert_lshift(null, "1.0");

trace("//undefined << \"1.0\"");
assert_lshift(undefined, "1.0");

trace("//\"\" << \"1.0\"");
assert_lshift("", "1.0");

trace("//\"str\" << \"1.0\"");
assert_lshift("str", "1.0");

trace("//\"true\" << \"1.0\"");
assert_lshift("true", "1.0");

trace("//\"false\" << \"1.0\"");
assert_lshift("false", "1.0");

trace("//0.0 << \"1.0\"");
assert_lshift(0.0, "1.0");

trace("//NaN << \"1.0\"");
assert_lshift(NaN, "1.0");

trace("//-0.0 << \"1.0\"");
assert_lshift(-0.0, "1.0");

trace("//Infinity << \"1.0\"");
assert_lshift(Infinity, "1.0");

trace("//1.0 << \"1.0\"");
assert_lshift(1.0, "1.0");

trace("//-1.0 << \"1.0\"");
assert_lshift(-1.0, "1.0");

trace("//0xFF1306 << \"1.0\"");
assert_lshift(0xFF1306, "1.0");

trace("//new Object() << \"1.0\"");
assert_lshift({}, "1.0");

trace("//\"0.0\" << \"1.0\"");
assert_lshift("0.0", "1.0");

trace("//\"NaN\" << \"1.0\"");
assert_lshift("NaN", "1.0");

trace("//\"-0.0\" << \"1.0\"");
assert_lshift("-0.0", "1.0");

trace("//\"Infinity\" << \"1.0\"");
assert_lshift("Infinity", "1.0");

trace("//\"1.0\" << \"1.0\"");
assert_lshift("1.0", "1.0");

trace("//\"-1.0\" << \"1.0\"");
assert_lshift("-1.0", "1.0");

trace("//\"0xFF1306\" << \"1.0\"");
assert_lshift("0xFF1306", "1.0");

trace("//true << \"-1.0\"");
assert_lshift(true, "-1.0");

trace("//false << \"-1.0\"");
assert_lshift(false, "-1.0");

trace("//null << \"-1.0\"");
assert_lshift(null, "-1.0");

trace("//undefined << \"-1.0\"");
assert_lshift(undefined, "-1.0");

trace("//\"\" << \"-1.0\"");
assert_lshift("", "-1.0");

trace("//\"str\" << \"-1.0\"");
assert_lshift("str", "-1.0");

trace("//\"true\" << \"-1.0\"");
assert_lshift("true", "-1.0");

trace("//\"false\" << \"-1.0\"");
assert_lshift("false", "-1.0");

trace("//0.0 << \"-1.0\"");
assert_lshift(0.0, "-1.0");

trace("//NaN << \"-1.0\"");
assert_lshift(NaN, "-1.0");

trace("//-0.0 << \"-1.0\"");
assert_lshift(-0.0, "-1.0");

trace("//Infinity << \"-1.0\"");
assert_lshift(Infinity, "-1.0");

trace("//1.0 << \"-1.0\"");
assert_lshift(1.0, "-1.0");

trace("//-1.0 << \"-1.0\"");
assert_lshift(-1.0, "-1.0");

trace("//0xFF1306 << \"-1.0\"");
assert_lshift(0xFF1306, "-1.0");

trace("//new Object() << \"-1.0\"");
assert_lshift({}, "-1.0");

trace("//\"0.0\" << \"-1.0\"");
assert_lshift("0.0", "-1.0");

trace("//\"NaN\" << \"-1.0\"");
assert_lshift("NaN", "-1.0");

trace("//\"-0.0\" << \"-1.0\"");
assert_lshift("-0.0", "-1.0");

trace("//\"Infinity\" << \"-1.0\"");
assert_lshift("Infinity", "-1.0");

trace("//\"1.0\" << \"-1.0\"");
assert_lshift("1.0", "-1.0");

trace("//\"-1.0\" << \"-1.0\"");
assert_lshift("-1.0", "-1.0");

trace("//\"0xFF1306\" << \"-1.0\"");
assert_lshift("0xFF1306", "-1.0");

trace("//true << \"0xFF1306\"");
assert_lshift(true, "0xFF1306");

trace("//false << \"0xFF1306\"");
assert_lshift(false, "0xFF1306");

trace("//null << \"0xFF1306\"");
assert_lshift(null, "0xFF1306");

trace("//undefined << \"0xFF1306\"");
assert_lshift(undefined, "0xFF1306");

trace("//\"\" << \"0xFF1306\"");
assert_lshift("", "0xFF1306");

trace("//\"str\" << \"0xFF1306\"");
assert_lshift("str", "0xFF1306");

trace("//\"true\" << \"0xFF1306\"");
assert_lshift("true", "0xFF1306");

trace("//\"false\" << \"0xFF1306\"");
assert_lshift("false", "0xFF1306");

trace("//0.0 << \"0xFF1306\"");
assert_lshift(0.0, "0xFF1306");

trace("//NaN << \"0xFF1306\"");
assert_lshift(NaN, "0xFF1306");

trace("//-0.0 << \"0xFF1306\"");
assert_lshift(-0.0, "0xFF1306");

trace("//Infinity << \"0xFF1306\"");
assert_lshift(Infinity, "0xFF1306");

trace("//1.0 << \"0xFF1306\"");
assert_lshift(1.0, "0xFF1306");

trace("//-1.0 << \"0xFF1306\"");
assert_lshift(-1.0, "0xFF1306");

trace("//0xFF1306 << \"0xFF1306\"");
assert_lshift(0xFF1306, "0xFF1306");

trace("//new Object() << \"0xFF1306\"");
assert_lshift({}, "0xFF1306");

trace("//\"0.0\" << \"0xFF1306\"");
assert_lshift("0.0", "0xFF1306");

trace("//\"NaN\" << \"0xFF1306\"");
assert_lshift("NaN", "0xFF1306");

trace("//\"-0.0\" << \"0xFF1306\"");
assert_lshift("-0.0", "0xFF1306");

trace("//\"Infinity\" << \"0xFF1306\"");
assert_lshift("Infinity", "0xFF1306");

trace("//\"1.0\" << \"0xFF1306\"");
assert_lshift("1.0", "0xFF1306");

trace("//\"-1.0\" << \"0xFF1306\"");
assert_lshift("-1.0", "0xFF1306");

trace("//\"0xFF1306\" << \"0xFF1306\"");
assert_lshift("0xFF1306", "0xFF1306");