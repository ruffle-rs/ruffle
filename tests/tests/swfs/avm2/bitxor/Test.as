package {
	public class Test {
	}
}

function assert_bitxor(val1, val2) {
	trace(val1 ^ val2);
}

trace("//true ^ true");
assert_bitxor(true, true);

trace("//false ^ true");
assert_bitxor(false, true);

trace("//null ^ true");
assert_bitxor(null, true);

trace("//undefined ^ true");
assert_bitxor(undefined, true);

trace("//\"\" ^ true");
assert_bitxor("", true);

trace("//\"str\" ^ true");
assert_bitxor("str", true);

trace("//\"true\" ^ true");
assert_bitxor("true", true);

trace("//\"false\" ^ true");
assert_bitxor("false", true);

trace("//0.0 ^ true");
assert_bitxor(0.0, true);

trace("//NaN ^ true");
assert_bitxor(NaN, true);

trace("//-0.0 ^ true");
assert_bitxor(-0.0, true);

trace("//Infinity ^ true");
assert_bitxor(Infinity, true);

trace("//1.0 ^ true");
assert_bitxor(1.0, true);

trace("//-1.0 ^ true");
assert_bitxor(-1.0, true);

trace("//0xFF1306 ^ true");
assert_bitxor(0xFF1306, true);

trace("//new Object() ^ true");
assert_bitxor({}, true);

trace("//\"0.0\" ^ true");
assert_bitxor("0.0", true);

trace("//\"NaN\" ^ true");
assert_bitxor("NaN", true);

trace("//\"-0.0\" ^ true");
assert_bitxor("-0.0", true);

trace("//\"Infinity\" ^ true");
assert_bitxor("Infinity", true);

trace("//\"1.0\" ^ true");
assert_bitxor("1.0", true);

trace("//\"-1.0\" ^ true");
assert_bitxor("-1.0", true);

trace("//\"0xFF1306\" ^ true");
assert_bitxor("0xFF1306", true);

trace("//true ^ false");
assert_bitxor(true, false);

trace("//false ^ false");
assert_bitxor(false, false);

trace("//null ^ false");
assert_bitxor(null, false);

trace("//undefined ^ false");
assert_bitxor(undefined, false);

trace("//\"\" ^ false");
assert_bitxor("", false);

trace("//\"str\" ^ false");
assert_bitxor("str", false);

trace("//\"true\" ^ false");
assert_bitxor("true", false);

trace("//\"false\" ^ false");
assert_bitxor("false", false);

trace("//0.0 ^ false");
assert_bitxor(0.0, false);

trace("//NaN ^ false");
assert_bitxor(NaN, false);

trace("//-0.0 ^ false");
assert_bitxor(-0.0, false);

trace("//Infinity ^ false");
assert_bitxor(Infinity, false);

trace("//1.0 ^ false");
assert_bitxor(1.0, false);

trace("//-1.0 ^ false");
assert_bitxor(-1.0, false);

trace("//0xFF1306 ^ false");
assert_bitxor(0xFF1306, false);

trace("//new Object() ^ false");
assert_bitxor({}, false);

trace("//\"0.0\" ^ false");
assert_bitxor("0.0", false);

trace("//\"NaN\" ^ false");
assert_bitxor("NaN", false);

trace("//\"-0.0\" ^ false");
assert_bitxor("-0.0", false);

trace("//\"Infinity\" ^ false");
assert_bitxor("Infinity", false);

trace("//\"1.0\" ^ false");
assert_bitxor("1.0", false);

trace("//\"-1.0\" ^ false");
assert_bitxor("-1.0", false);

trace("//\"0xFF1306\" ^ false");
assert_bitxor("0xFF1306", false);
trace("//true ^ null");
assert_bitxor(true, null);

trace("//false ^ null");
assert_bitxor(false, null);

trace("//null ^ null");
assert_bitxor(null, null);

trace("//undefined ^ null");
assert_bitxor(undefined, null);

trace("//\"\" ^ null");
assert_bitxor("", null);

trace("//\"str\" ^ null");
assert_bitxor("str", null);

trace("//\"true\" ^ null");
assert_bitxor("true", null);

trace("//\"false\" ^ null");
assert_bitxor("false", null);

trace("//0.0 ^ null");
assert_bitxor(0.0, null);

trace("//NaN ^ null");
assert_bitxor(NaN, null);

trace("//-0.0 ^ null");
assert_bitxor(-0.0, null);

trace("//Infinity ^ null");
assert_bitxor(Infinity, null);

trace("//1.0 ^ null");
assert_bitxor(1.0, null);

trace("//-1.0 ^ null");
assert_bitxor(-1.0, null);

trace("//0xFF1306 ^ null");
assert_bitxor(0xFF1306, null);

trace("//new Object() ^ null");
assert_bitxor({}, null);

trace("//\"0.0\" ^ null");
assert_bitxor("0.0", null);

trace("//\"NaN\" ^ null");
assert_bitxor("NaN", null);

trace("//\"-0.0\" ^ null");
assert_bitxor("-0.0", null);

trace("//\"Infinity\" ^ null");
assert_bitxor("Infinity", null);

trace("//\"1.0\" ^ null");
assert_bitxor("1.0", null);

trace("//\"-1.0\" ^ null");
assert_bitxor("-1.0", null);

trace("//\"0xFF1306\" ^ null");
assert_bitxor("0xFF1306", null);

trace("//true ^ undefined");
assert_bitxor(true, undefined);

trace("//false ^ undefined");
assert_bitxor(false, undefined);

trace("//null ^ undefined");
assert_bitxor(null, undefined);

trace("//undefined ^ undefined");
assert_bitxor(undefined, undefined);

trace("//\"\" ^ undefined");
assert_bitxor("", undefined);

trace("//\"str\" ^ undefined");
assert_bitxor("str", undefined);

trace("//\"true\" ^ undefined");
assert_bitxor("true", undefined);

trace("//\"false\" ^ undefined");
assert_bitxor("false", undefined);

trace("//0.0 ^ undefined");
assert_bitxor(0.0, undefined);

trace("//NaN ^ undefined");
assert_bitxor(NaN, undefined);

trace("//-0.0 ^ undefined");
assert_bitxor(-0.0, undefined);

trace("//Infinity ^ undefined");
assert_bitxor(Infinity, undefined);

trace("//1.0 ^ undefined");
assert_bitxor(1.0, undefined);

trace("//-1.0 ^ undefined");
assert_bitxor(-1.0, undefined);

trace("//0xFF1306 ^ undefined");
assert_bitxor(0xFF1306, undefined);

trace("//new Object() ^ undefined");
assert_bitxor({}, undefined);

trace("//\"0.0\" ^ undefined");
assert_bitxor("0.0", undefined);

trace("//\"NaN\" ^ undefined");
assert_bitxor("NaN", undefined);

trace("//\"-0.0\" ^ undefined");
assert_bitxor("-0.0", undefined);

trace("//\"Infinity\" ^ undefined");
assert_bitxor("Infinity", undefined);

trace("//\"1.0\" ^ undefined");
assert_bitxor("1.0", undefined);

trace("//\"-1.0\" ^ undefined");
assert_bitxor("-1.0", undefined);

trace("//\"0xFF1306\" ^ undefined");
assert_bitxor("0xFF1306", undefined);

trace("//true ^ \"\"");
assert_bitxor(true, "");

trace("//false ^ \"\"");
assert_bitxor(false, "");

trace("//null ^ \"\"");
assert_bitxor(null, "");

trace("//undefined ^ \"\"");
assert_bitxor(undefined, "");

trace("//\"\" ^ \"\"");
assert_bitxor("", "");

trace("//\"str\" ^ \"\"");
assert_bitxor("str", "");

trace("//\"true\" ^ \"\"");
assert_bitxor("true", "");

trace("//\"false\" ^ \"\"");
assert_bitxor("false", "");

trace("//0.0 ^ \"\"");
assert_bitxor(0.0, "");

trace("//NaN ^ \"\"");
assert_bitxor(NaN, "");

trace("//-0.0 ^ \"\"");
assert_bitxor(-0.0, "");

trace("//Infinity ^ \"\"");
assert_bitxor(Infinity, "");

trace("//1.0 ^ \"\"");
assert_bitxor(1.0, "");

trace("//-1.0 ^ \"\"");
assert_bitxor(-1.0, "");

trace("//0xFF1306 ^ \"\"");
assert_bitxor(0xFF1306, "");

trace("//new Object() ^ \"\"");
assert_bitxor({}, "");

trace("//\"0.0\" ^ \"\"");
assert_bitxor("0.0", "");

trace("//\"NaN\" ^ \"\"");
assert_bitxor("NaN", "");

trace("//\"-0.0\" ^ \"\"");
assert_bitxor("-0.0", "");

trace("//\"Infinity\" ^ \"\"");
assert_bitxor("Infinity", "");

trace("//\"1.0\" ^ \"\"");
assert_bitxor("1.0", "");

trace("//\"-1.0\" ^ \"\"");
assert_bitxor("-1.0", "");

trace("//\"0xFF1306\" ^ \"\"");
assert_bitxor("0xFF1306", "");

trace("//true ^ \"str\"");
assert_bitxor(true, "str");

trace("//false ^ \"str\"");
assert_bitxor(false, "str");

trace("//null ^ \"str\"");
assert_bitxor(null, "str");

trace("//undefined ^ \"str\"");
assert_bitxor(undefined, "str");

trace("//\"\" ^ \"str\"");
assert_bitxor("", "str");

trace("//\"str\" ^ \"str\"");
assert_bitxor("str", "str");

trace("//\"true\" ^ \"str\"");
assert_bitxor("true", "str");

trace("//\"false\" ^ \"str\"");
assert_bitxor("false", "str");

trace("//0.0 ^ \"str\"");
assert_bitxor(0.0, "str");

trace("//NaN ^ \"str\"");
assert_bitxor(NaN, "str");

trace("//-0.0 ^ \"str\"");
assert_bitxor(-0.0, "str");

trace("//Infinity ^ \"str\"");
assert_bitxor(Infinity, "str");

trace("//1.0 ^ \"str\"");
assert_bitxor(1.0, "str");

trace("//-1.0 ^ \"str\"");
assert_bitxor(-1.0, "str");

trace("//0xFF1306 ^ \"str\"");
assert_bitxor(0xFF1306, "str");

trace("//new Object() ^ \"str\"");
assert_bitxor({}, "str");

trace("//\"0.0\" ^ \"str\"");
assert_bitxor("0.0", "str");

trace("//\"NaN\" ^ \"str\"");
assert_bitxor("NaN", "str");

trace("//\"-0.0\" ^ \"str\"");
assert_bitxor("-0.0", "str");

trace("//\"Infinity\" ^ \"str\"");
assert_bitxor("Infinity", "str");

trace("//\"1.0\" ^ \"str\"");
assert_bitxor("1.0", "str");

trace("//\"-1.0\" ^ \"str\"");
assert_bitxor("-1.0", "str");

trace("//\"0xFF1306\" ^ \"str\"");
assert_bitxor("0xFF1306", "str");

trace("//true ^ \"true\"");
assert_bitxor(true, "true");

trace("//false ^ \"true\"");
assert_bitxor(false, "true");

trace("//null ^ \"true\"");
assert_bitxor(null, "true");

trace("//undefined ^ \"true\"");
assert_bitxor(undefined, "true");

trace("//\"\" ^ \"true\"");
assert_bitxor("", "true");

trace("//\"str\" ^ \"true\"");
assert_bitxor("str", "true");

trace("//\"true\" ^ \"true\"");
assert_bitxor("true", "true");

trace("//\"false\" ^ \"true\"");
assert_bitxor("false", "true");

trace("//0.0 ^ \"true\"");
assert_bitxor(0.0, "true");

trace("//NaN ^ \"true\"");
assert_bitxor(NaN, "true");

trace("//-0.0 ^ \"true\"");
assert_bitxor(-0.0, "true");

trace("//Infinity ^ \"true\"");
assert_bitxor(Infinity, "true");

trace("//1.0 ^ \"true\"");
assert_bitxor(1.0, "true");

trace("//-1.0 ^ \"true\"");
assert_bitxor(-1.0, "true");

trace("//0xFF1306 ^ \"true\"");
assert_bitxor(0xFF1306, "true");

trace("//new Object() ^ \"true\"");
assert_bitxor({}, "true");

trace("//\"0.0\" ^ \"true\"");
assert_bitxor("0.0", "true");

trace("//\"NaN\" ^ \"true\"");
assert_bitxor("NaN", "true");

trace("//\"-0.0\" ^ \"true\"");
assert_bitxor("-0.0", "true");

trace("//\"Infinity\" ^ \"true\"");
assert_bitxor("Infinity", "true");

trace("//\"1.0\" ^ \"true\"");
assert_bitxor("1.0", "true");

trace("//\"-1.0\" ^ \"true\"");
assert_bitxor("-1.0", "true");

trace("//\"0xFF1306\" ^ \"true\"");
assert_bitxor("0xFF1306", "true");

trace("//true ^ \"false\"");
assert_bitxor(true, "false");

trace("//false ^ \"false\"");
assert_bitxor(false, "false");

trace("//null ^ \"false\"");
assert_bitxor(null, "false");

trace("//undefined ^ \"false\"");
assert_bitxor(undefined, "false");

trace("//\"\" ^ \"false\"");
assert_bitxor("", "false");

trace("//\"str\" ^ \"false\"");
assert_bitxor("str", "false");

trace("//\"true\" ^ \"false\"");
assert_bitxor("true", "false");

trace("//\"false\" ^ \"false\"");
assert_bitxor("false", "false");

trace("//0.0 ^ \"false\"");
assert_bitxor(0.0, "false");

trace("//NaN ^ \"false\"");
assert_bitxor(NaN, "false");

trace("//-0.0 ^ \"false\"");
assert_bitxor(-0.0, "false");

trace("//Infinity ^ \"false\"");
assert_bitxor(Infinity, "false");

trace("//1.0 ^ \"false\"");
assert_bitxor(1.0, "false");

trace("//-1.0 ^ \"false\"");
assert_bitxor(-1.0, "false");

trace("//0xFF1306 ^ \"false\"");
assert_bitxor(0xFF1306, "false");

trace("//new Object() ^ \"false\"");
assert_bitxor({}, "false");

trace("//\"0.0\" ^ \"false\"");
assert_bitxor("0.0", "false");

trace("//\"NaN\" ^ \"false\"");
assert_bitxor("NaN", "false");

trace("//\"-0.0\" ^ \"false\"");
assert_bitxor("-0.0", "false");

trace("//\"Infinity\" ^ \"false\"");
assert_bitxor("Infinity", "false");

trace("//\"1.0\" ^ \"false\"");
assert_bitxor("1.0", "false");

trace("//\"-1.0\" ^ \"false\"");
assert_bitxor("-1.0", "false");

trace("//\"0xFF1306\" ^ \"false\"");
assert_bitxor("0xFF1306", "false");

trace("//true ^ 0.0");
assert_bitxor(true, 0.0);

trace("//false ^ 0.0");
assert_bitxor(false, 0.0);

trace("//null ^ 0.0");
assert_bitxor(null, 0.0);

trace("//undefined ^ 0.0");
assert_bitxor(undefined, 0.0);

trace("//\"\" ^ 0.0");
assert_bitxor("", 0.0);

trace("//\"str\" ^ 0.0");
assert_bitxor("str", 0.0);

trace("//\"true\" ^ 0.0");
assert_bitxor("true", 0.0);

trace("//\"false\" ^ 0.0");
assert_bitxor("false", 0.0);

trace("//0.0 ^ 0.0");
assert_bitxor(0.0, 0.0);

trace("//NaN ^ 0.0");
assert_bitxor(NaN, 0.0);

trace("//-0.0 ^ 0.0");
assert_bitxor(-0.0, 0.0);

trace("//Infinity ^ 0.0");
assert_bitxor(Infinity, 0.0);

trace("//1.0 ^ 0.0");
assert_bitxor(1.0, 0.0);

trace("//-1.0 ^ 0.0");
assert_bitxor(-1.0, 0.0);

trace("//0xFF1306 ^ 0.0");
assert_bitxor(0xFF1306, 0.0);

trace("//new Object() ^ 0.0");
assert_bitxor({}, 0.0);

trace("//\"0.0\" ^ 0.0");
assert_bitxor("0.0", 0.0);

trace("//\"NaN\" ^ 0.0");
assert_bitxor("NaN", 0.0);

trace("//\"-0.0\" ^ 0.0");
assert_bitxor("-0.0", 0.0);

trace("//\"Infinity\" ^ 0.0");
assert_bitxor("Infinity", 0.0);

trace("//\"1.0\" ^ 0.0");
assert_bitxor("1.0", 0.0);

trace("//\"-1.0\" ^ 0.0");
assert_bitxor("-1.0", 0.0);

trace("//\"0xFF1306\" ^ 0.0");
assert_bitxor("0xFF1306", 0.0);

trace("//true ^ NaN");
assert_bitxor(true, NaN);

trace("//false ^ NaN");
assert_bitxor(false, NaN);

trace("//null ^ NaN");
assert_bitxor(null, NaN);

trace("//undefined ^ NaN");
assert_bitxor(undefined, NaN);

trace("//\"\" ^ NaN");
assert_bitxor("", NaN);

trace("//\"str\" ^ NaN");
assert_bitxor("str", NaN);

trace("//\"true\" ^ NaN");
assert_bitxor("true", NaN);

trace("//\"false\" ^ NaN");
assert_bitxor("false", NaN);

trace("//0.0 ^ NaN");
assert_bitxor(0.0, NaN);

trace("//NaN ^ NaN");
assert_bitxor(NaN, NaN);

trace("//-0.0 ^ NaN");
assert_bitxor(-0.0, NaN);

trace("//Infinity ^ NaN");
assert_bitxor(Infinity, NaN);

trace("//1.0 ^ NaN");
assert_bitxor(1.0, NaN);

trace("//-1.0 ^ NaN");
assert_bitxor(-1.0, NaN);

trace("//0xFF1306 ^ NaN");
assert_bitxor(0xFF1306, NaN);

trace("//new Object() ^ NaN");
assert_bitxor({}, NaN);

trace("//\"0.0\" ^ NaN");
assert_bitxor("0.0", NaN);

trace("//\"NaN\" ^ NaN");
assert_bitxor("NaN", NaN);

trace("//\"-0.0\" ^ NaN");
assert_bitxor("-0.0", NaN);

trace("//\"Infinity\" ^ NaN");
assert_bitxor("Infinity", NaN);

trace("//\"1.0\" ^ NaN");
assert_bitxor("1.0", NaN);

trace("//\"-1.0\" ^ NaN");
assert_bitxor("-1.0", NaN);

trace("//\"0xFF1306\" ^ NaN");
assert_bitxor("0xFF1306", NaN);

trace("//true ^ -0.0");
assert_bitxor(true, -0.0);

trace("//false ^ -0.0");
assert_bitxor(false, -0.0);

trace("//null ^ -0.0");
assert_bitxor(null, -0.0);

trace("//undefined ^ -0.0");
assert_bitxor(undefined, -0.0);

trace("//\"\" ^ -0.0");
assert_bitxor("", -0.0);

trace("//\"str\" ^ -0.0");
assert_bitxor("str", -0.0);

trace("//\"true\" ^ -0.0");
assert_bitxor("true", -0.0);

trace("//\"false\" ^ -0.0");
assert_bitxor("false", -0.0);

trace("//0.0 ^ -0.0");
assert_bitxor(0.0, -0.0);

trace("//NaN ^ -0.0");
assert_bitxor(NaN, -0.0);

trace("//-0.0 ^ -0.0");
assert_bitxor(-0.0, -0.0);

trace("//Infinity ^ -0.0");
assert_bitxor(Infinity, -0.0);

trace("//1.0 ^ -0.0");
assert_bitxor(1.0, -0.0);

trace("//-1.0 ^ -0.0");
assert_bitxor(-1.0, -0.0);

trace("//0xFF1306 ^ -0.0");
assert_bitxor(0xFF1306, -0.0);

trace("//new Object() ^ -0.0");
assert_bitxor({}, -0.0);

trace("//\"0.0\" ^ -0.0");
assert_bitxor("0.0", -0.0);

trace("//\"NaN\" ^ -0.0");
assert_bitxor("NaN", -0.0);

trace("//\"-0.0\" ^ -0.0");
assert_bitxor("-0.0", -0.0);

trace("//\"Infinity\" ^ -0.0");
assert_bitxor("Infinity", -0.0);

trace("//\"1.0\" ^ -0.0");
assert_bitxor("1.0", -0.0);

trace("//\"-1.0\" ^ -0.0");
assert_bitxor("-1.0", -0.0);

trace("//\"0xFF1306\" ^ -0.0");
assert_bitxor("0xFF1306", -0.0);

trace("//true ^ Infinity");
assert_bitxor(true, Infinity);

trace("//false ^ Infinity");
assert_bitxor(false, Infinity);

trace("//null ^ Infinity");
assert_bitxor(null, Infinity);

trace("//undefined ^ Infinity");
assert_bitxor(undefined, Infinity);

trace("//\"\" ^ Infinity");
assert_bitxor("", Infinity);

trace("//\"str\" ^ Infinity");
assert_bitxor("str", Infinity);

trace("//\"true\" ^ Infinity");
assert_bitxor("true", Infinity);

trace("//\"false\" ^ Infinity");
assert_bitxor("false", Infinity);

trace("//0.0 ^ Infinity");
assert_bitxor(0.0, Infinity);

trace("//NaN ^ Infinity");
assert_bitxor(NaN, Infinity);

trace("//-0.0 ^ Infinity");
assert_bitxor(-0.0, Infinity);

trace("//Infinity ^ Infinity");
assert_bitxor(Infinity, Infinity);

trace("//1.0 ^ Infinity");
assert_bitxor(1.0, Infinity);

trace("//-1.0 ^ Infinity");
assert_bitxor(-1.0, Infinity);

trace("//0xFF1306 ^ Infinity");
assert_bitxor(0xFF1306, Infinity);

trace("//new Object() ^ Infinity");
assert_bitxor({}, Infinity);

trace("//\"0.0\" ^ Infinity");
assert_bitxor("0.0", Infinity);

trace("//\"NaN\" ^ Infinity");
assert_bitxor("NaN", Infinity);

trace("//\"-0.0\" ^ Infinity");
assert_bitxor("-0.0", Infinity);

trace("//\"Infinity\" ^ Infinity");
assert_bitxor("Infinity", Infinity);

trace("//\"1.0\" ^ Infinity");
assert_bitxor("1.0", Infinity);

trace("//\"-1.0\" ^ Infinity");
assert_bitxor("-1.0", Infinity);

trace("//\"0xFF1306\" ^ Infinity");
assert_bitxor("0xFF1306", Infinity);

trace("//true ^ 1.0");
assert_bitxor(true, 1.0);

trace("//false ^ 1.0");
assert_bitxor(false, 1.0);

trace("//null ^ 1.0");
assert_bitxor(null, 1.0);

trace("//undefined ^ 1.0");
assert_bitxor(undefined, 1.0);

trace("//\"\" ^ 1.0");
assert_bitxor("", 1.0);

trace("//\"str\" ^ 1.0");
assert_bitxor("str", 1.0);

trace("//\"true\" ^ 1.0");
assert_bitxor("true", 1.0);

trace("//\"false\" ^ 1.0");
assert_bitxor("false", 1.0);

trace("//0.0 ^ 1.0");
assert_bitxor(0.0, 1.0);

trace("//NaN ^ 1.0");
assert_bitxor(NaN, 1.0);

trace("//-0.0 ^ 1.0");
assert_bitxor(-0.0, 1.0);

trace("//Infinity ^ 1.0");
assert_bitxor(Infinity, 1.0);

trace("//1.0 ^ 1.0");
assert_bitxor(1.0, 1.0);

trace("//-1.0 ^ 1.0");
assert_bitxor(-1.0, 1.0);

trace("//0xFF1306 ^ 1.0");
assert_bitxor(0xFF1306, 1.0);

trace("//new Object() ^ 1.0");
assert_bitxor({}, 1.0);

trace("//\"0.0\" ^ 1.0");
assert_bitxor("0.0", 1.0);

trace("//\"NaN\" ^ 1.0");
assert_bitxor("NaN", 1.0);

trace("//\"-0.0\" ^ 1.0");
assert_bitxor("-0.0", 1.0);

trace("//\"Infinity\" ^ 1.0");
assert_bitxor("Infinity", 1.0);

trace("//\"1.0\" ^ 1.0");
assert_bitxor("1.0", 1.0);

trace("//\"-1.0\" ^ 1.0");
assert_bitxor("-1.0", 1.0);

trace("//\"0xFF1306\" ^ 1.0");
assert_bitxor("0xFF1306", 1.0);

trace("//true ^ -1.0");
assert_bitxor(true, -1.0);

trace("//false ^ -1.0");
assert_bitxor(false, -1.0);

trace("//null ^ -1.0");
assert_bitxor(null, -1.0);

trace("//undefined ^ -1.0");
assert_bitxor(undefined, -1.0);

trace("//\"\" ^ -1.0");
assert_bitxor("", -1.0);

trace("//\"str\" ^ -1.0");
assert_bitxor("str", -1.0);

trace("//\"true\" ^ -1.0");
assert_bitxor("true", -1.0);

trace("//\"false\" ^ -1.0");
assert_bitxor("false", -1.0);

trace("//0.0 ^ -1.0");
assert_bitxor(0.0, -1.0);

trace("//NaN ^ -1.0");
assert_bitxor(NaN, -1.0);

trace("//-0.0 ^ -1.0");
assert_bitxor(-0.0, -1.0);

trace("//Infinity ^ -1.0");
assert_bitxor(Infinity, -1.0);

trace("//1.0 ^ -1.0");
assert_bitxor(1.0, -1.0);

trace("//-1.0 ^ -1.0");
assert_bitxor(-1.0, -1.0);

trace("//0xFF1306 ^ -1.0");
assert_bitxor(0xFF1306, -1.0);

trace("//new Object() ^ -1.0");
assert_bitxor({}, -1.0);

trace("//\"0.0\" ^ -1.0");
assert_bitxor("0.0", -1.0);

trace("//\"NaN\" ^ -1.0");
assert_bitxor("NaN", -1.0);

trace("//\"-0.0\" ^ -1.0");
assert_bitxor("-0.0", -1.0);

trace("//\"Infinity\" ^ -1.0");
assert_bitxor("Infinity", -1.0);

trace("//\"1.0\" ^ -1.0");
assert_bitxor("1.0", -1.0);

trace("//\"-1.0\" ^ -1.0");
assert_bitxor("-1.0", -1.0);

trace("//\"0xFF1306\" ^ -1.0");
assert_bitxor("0xFF1306", -1.0);

trace("//true ^ 0xFF1306");
assert_bitxor(true, 0xFF1306);

trace("//false ^ 0xFF1306");
assert_bitxor(false, 0xFF1306);

trace("//null ^ 0xFF1306");
assert_bitxor(null, 0xFF1306);

trace("//undefined ^ 0xFF1306");
assert_bitxor(undefined, 0xFF1306);

trace("//\"\" ^ 0xFF1306");
assert_bitxor("", 0xFF1306);

trace("//\"str\" ^ 0xFF1306");
assert_bitxor("str", 0xFF1306);

trace("//\"true\" ^ 0xFF1306");
assert_bitxor("true", 0xFF1306);

trace("//\"false\" ^ 0xFF1306");
assert_bitxor("false", 0xFF1306);

trace("//0.0 ^ 0xFF1306");
assert_bitxor(0.0, 0xFF1306);

trace("//NaN ^ 0xFF1306");
assert_bitxor(NaN, 0xFF1306);

trace("//-0.0 ^ 0xFF1306");
assert_bitxor(-0.0, 0xFF1306);

trace("//Infinity ^ 0xFF1306");
assert_bitxor(Infinity, 0xFF1306);

trace("//1.0 ^ 0xFF1306");
assert_bitxor(1.0, 0xFF1306);

trace("//-1.0 ^ 0xFF1306");
assert_bitxor(-1.0, 0xFF1306);

trace("//0xFF1306 ^ 0xFF1306");
assert_bitxor(0xFF1306, 0xFF1306);

trace("//new Object() ^ 0xFF1306");
assert_bitxor({}, 0xFF1306);

trace("//\"0.0\" ^ 0xFF1306");
assert_bitxor("0.0", 0xFF1306);

trace("//\"NaN\" ^ 0xFF1306");
assert_bitxor("NaN", 0xFF1306);

trace("//\"-0.0\" ^ 0xFF1306");
assert_bitxor("-0.0", 0xFF1306);

trace("//\"Infinity\" ^ 0xFF1306");
assert_bitxor("Infinity", 0xFF1306);

trace("//\"1.0\" ^ 0xFF1306");
assert_bitxor("1.0", 0xFF1306);

trace("//\"-1.0\" ^ 0xFF1306");
assert_bitxor("-1.0", 0xFF1306);

trace("//\"0xFF1306\" ^ 0xFF1306");
assert_bitxor("0xFF1306", 0xFF1306);

trace("//true ^ new Object()");
assert_bitxor(true, {});

trace("//false ^ new Object()");
assert_bitxor(false, {});

trace("//null ^ new Object()");
assert_bitxor(null, {});

trace("//undefined ^ new Object()");
assert_bitxor(undefined, {});

trace("//\"\" ^ new Object()");
assert_bitxor("", {});

trace("//\"str\" ^ new Object()");
assert_bitxor("str", {});

trace("//\"true\" ^ new Object()");
assert_bitxor("true", {});

trace("//\"false\" ^ new Object()");
assert_bitxor("false", {});

trace("//0.0 ^ new Object()");
assert_bitxor(0.0, {});

trace("//NaN ^ new Object()");
assert_bitxor(NaN, {});

trace("//-0.0 ^ new Object()");
assert_bitxor(-0.0, {});

trace("//Infinity ^ new Object()");
assert_bitxor(Infinity, {});

trace("//1.0 ^ new Object()");
assert_bitxor(1.0, {});

trace("//-1.0 ^ new Object()");
assert_bitxor(-1.0, {});

trace("//0xFF1306 ^ new Object()");
assert_bitxor(0xFF1306, {});

trace("//new Object() ^ new Object()");
assert_bitxor({}, {});

trace("//\"0.0\" ^ new Object()");
assert_bitxor("0.0", {});

trace("//\"NaN\" ^ new Object()");
assert_bitxor("NaN", {});

trace("//\"-0.0\" ^ new Object()");
assert_bitxor("-0.0", {});

trace("//\"Infinity\" ^ new Object()");
assert_bitxor("Infinity", {});

trace("//\"1.0\" ^ new Object()");
assert_bitxor("1.0", {});

trace("//\"-1.0\" ^ new Object()");
assert_bitxor("-1.0", {});

trace("//\"0xFF1306\" ^ new Object()");
assert_bitxor("0xFF1306", {});

trace("//true ^ \"0.0\"");
assert_bitxor(true, "0.0");

trace("//false ^ \"0.0\"");
assert_bitxor(false, "0.0");

trace("//null ^ \"0.0\"");
assert_bitxor(null, "0.0");

trace("//undefined ^ \"0.0\"");
assert_bitxor(undefined, "0.0");

trace("//\"\" ^ \"0.0\"");
assert_bitxor("", "0.0");

trace("//\"str\" ^ \"0.0\"");
assert_bitxor("str", "0.0");

trace("//\"true\" ^ \"0.0\"");
assert_bitxor("true", "0.0");

trace("//\"false\" ^ \"0.0\"");
assert_bitxor("false", "0.0");

trace("//0.0 ^ \"0.0\"");
assert_bitxor(0.0, "0.0");

trace("//NaN ^ \"0.0\"");
assert_bitxor(NaN, "0.0");

trace("//-0.0 ^ \"0.0\"");
assert_bitxor(-0.0, "0.0");

trace("//Infinity ^ \"0.0\"");
assert_bitxor(Infinity, "0.0");

trace("//1.0 ^ \"0.0\"");
assert_bitxor(1.0, "0.0");

trace("//-1.0 ^ \"0.0\"");
assert_bitxor(-1.0, "0.0");

trace("//0xFF1306 ^ \"0.0\"");
assert_bitxor(0xFF1306, "0.0");

trace("//new Object() ^ \"0.0\"");
assert_bitxor({}, "0.0");

trace("//\"0.0\" ^ \"0.0\"");
assert_bitxor("0.0", "0.0");

trace("//\"NaN\" ^ \"0.0\"");
assert_bitxor("NaN", "0.0");

trace("//\"-0.0\" ^ \"0.0\"");
assert_bitxor("-0.0", "0.0");

trace("//\"Infinity\" ^ \"0.0\"");
assert_bitxor("Infinity", "0.0");

trace("//\"1.0\" ^ \"0.0\"");
assert_bitxor("1.0", "0.0");

trace("//\"-1.0\" ^ \"0.0\"");
assert_bitxor("-1.0", "0.0");

trace("//\"0xFF1306\" ^ \"0.0\"");
assert_bitxor("0xFF1306", "0.0");

trace("//true ^ \"NaN\"");
assert_bitxor(true, "NaN");

trace("//false ^ \"NaN\"");
assert_bitxor(false, "NaN");

trace("//null ^ \"NaN\"");
assert_bitxor(null, "NaN");

trace("//undefined ^ \"NaN\"");
assert_bitxor(undefined, "NaN");

trace("//\"\" ^ \"NaN\"");
assert_bitxor("", "NaN");

trace("//\"str\" ^ \"NaN\"");
assert_bitxor("str", "NaN");

trace("//\"true\" ^ \"NaN\"");
assert_bitxor("true", "NaN");

trace("//\"false\" ^ \"NaN\"");
assert_bitxor("false", "NaN");

trace("//0.0 ^ \"NaN\"");
assert_bitxor(0.0, "NaN");

trace("//NaN ^ \"NaN\"");
assert_bitxor(NaN, "NaN");

trace("//-0.0 ^ \"NaN\"");
assert_bitxor(-0.0, "NaN");

trace("//Infinity ^ \"NaN\"");
assert_bitxor(Infinity, "NaN");

trace("//1.0 ^ \"NaN\"");
assert_bitxor(1.0, "NaN");

trace("//-1.0 ^ \"NaN\"");
assert_bitxor(-1.0, "NaN");

trace("//0xFF1306 ^ \"NaN\"");
assert_bitxor(0xFF1306, "NaN");

trace("//new Object() ^ \"NaN\"");
assert_bitxor({}, "NaN");

trace("//\"0.0\" ^ \"NaN\"");
assert_bitxor("0.0", "NaN");

trace("//\"NaN\" ^ \"NaN\"");
assert_bitxor("NaN", "NaN");

trace("//\"-0.0\" ^ \"NaN\"");
assert_bitxor("-0.0", "NaN");

trace("//\"Infinity\" ^ \"NaN\"");
assert_bitxor("Infinity", "NaN");

trace("//\"1.0\" ^ \"NaN\"");
assert_bitxor("1.0", "NaN");

trace("//\"-1.0\" ^ \"NaN\"");
assert_bitxor("-1.0", "NaN");

trace("//\"0xFF1306\" ^ \"NaN\"");
assert_bitxor("0xFF1306", "NaN");

trace("//true ^ \"-0.0\"");
assert_bitxor(true, "-0.0");

trace("//false ^ \"-0.0\"");
assert_bitxor(false, "-0.0");

trace("//null ^ \"-0.0\"");
assert_bitxor(null, "-0.0");

trace("//undefined ^ \"-0.0\"");
assert_bitxor(undefined, "-0.0");

trace("//\"\" ^ \"-0.0\"");
assert_bitxor("", "-0.0");

trace("//\"str\" ^ \"-0.0\"");
assert_bitxor("str", "-0.0");

trace("//\"true\" ^ \"-0.0\"");
assert_bitxor("true", "-0.0");

trace("//\"false\" ^ \"-0.0\"");
assert_bitxor("false", "-0.0");

trace("//0.0 ^ \"-0.0\"");
assert_bitxor(0.0, "-0.0");

trace("//NaN ^ \"-0.0\"");
assert_bitxor(NaN, "-0.0");

trace("//-0.0 ^ \"-0.0\"");
assert_bitxor(-0.0, "-0.0");

trace("//Infinity ^ \"-0.0\"");
assert_bitxor(Infinity, "-0.0");

trace("//1.0 ^ \"-0.0\"");
assert_bitxor(1.0, "-0.0");

trace("//-1.0 ^ \"-0.0\"");
assert_bitxor(-1.0, "-0.0");

trace("//0xFF1306 ^ \"-0.0\"");
assert_bitxor(0xFF1306, "-0.0");

trace("//new Object() ^ \"-0.0\"");
assert_bitxor({}, "-0.0");

trace("//\"0.0\" ^ \"-0.0\"");
assert_bitxor("0.0", "-0.0");

trace("//\"NaN\" ^ \"-0.0\"");
assert_bitxor("NaN", "-0.0");

trace("//\"-0.0\" ^ \"-0.0\"");
assert_bitxor("-0.0", "-0.0");

trace("//\"Infinity\" ^ \"-0.0\"");
assert_bitxor("Infinity", "-0.0");

trace("//\"1.0\" ^ \"-0.0\"");
assert_bitxor("1.0", "-0.0");

trace("//\"-1.0\" ^ \"-0.0\"");
assert_bitxor("-1.0", "-0.0");

trace("//\"0xFF1306\" ^ \"-0.0\"");
assert_bitxor("0xFF1306", "-0.0");

trace("//true ^ \"Infinity\"");
assert_bitxor(true, "Infinity");

trace("//false ^ \"Infinity\"");
assert_bitxor(false, "Infinity");

trace("//null ^ \"Infinity\"");
assert_bitxor(null, "Infinity");

trace("//undefined ^ \"Infinity\"");
assert_bitxor(undefined, "Infinity");

trace("//\"\" ^ \"Infinity\"");
assert_bitxor("", "Infinity");

trace("//\"str\" ^ \"Infinity\"");
assert_bitxor("str", "Infinity");

trace("//\"true\" ^ \"Infinity\"");
assert_bitxor("true", "Infinity");

trace("//\"false\" ^ \"Infinity\"");
assert_bitxor("false", "Infinity");

trace("//0.0 ^ \"Infinity\"");
assert_bitxor(0.0, "Infinity");

trace("//NaN ^ \"Infinity\"");
assert_bitxor(NaN, "Infinity");

trace("//-0.0 ^ \"Infinity\"");
assert_bitxor(-0.0, "Infinity");

trace("//Infinity ^ \"Infinity\"");
assert_bitxor(Infinity, "Infinity");

trace("//1.0 ^ \"Infinity\"");
assert_bitxor(1.0, "Infinity");

trace("//-1.0 ^ \"Infinity\"");
assert_bitxor(-1.0, "Infinity");

trace("//0xFF1306 ^ \"Infinity\"");
assert_bitxor(0xFF1306, "Infinity");

trace("//new Object() ^ \"Infinity\"");
assert_bitxor({}, "Infinity");

trace("//\"0.0\" ^ \"Infinity\"");
assert_bitxor("0.0", "Infinity");

trace("//\"NaN\" ^ \"Infinity\"");
assert_bitxor("NaN", "Infinity");

trace("//\"-0.0\" ^ \"Infinity\"");
assert_bitxor("-0.0", "Infinity");

trace("//\"Infinity\" ^ \"Infinity\"");
assert_bitxor("Infinity", "Infinity");

trace("//\"1.0\" ^ \"Infinity\"");
assert_bitxor("1.0", "Infinity");

trace("//\"-1.0\" ^ \"Infinity\"");
assert_bitxor("-1.0", "Infinity");

trace("//\"0xFF1306\" ^ \"Infinity\"");
assert_bitxor("0xFF1306", "Infinity");

trace("//true ^ \"1.0\"");
assert_bitxor(true, "1.0");

trace("//false ^ \"1.0\"");
assert_bitxor(false, "1.0");

trace("//null ^ \"1.0\"");
assert_bitxor(null, "1.0");

trace("//undefined ^ \"1.0\"");
assert_bitxor(undefined, "1.0");

trace("//\"\" ^ \"1.0\"");
assert_bitxor("", "1.0");

trace("//\"str\" ^ \"1.0\"");
assert_bitxor("str", "1.0");

trace("//\"true\" ^ \"1.0\"");
assert_bitxor("true", "1.0");

trace("//\"false\" ^ \"1.0\"");
assert_bitxor("false", "1.0");

trace("//0.0 ^ \"1.0\"");
assert_bitxor(0.0, "1.0");

trace("//NaN ^ \"1.0\"");
assert_bitxor(NaN, "1.0");

trace("//-0.0 ^ \"1.0\"");
assert_bitxor(-0.0, "1.0");

trace("//Infinity ^ \"1.0\"");
assert_bitxor(Infinity, "1.0");

trace("//1.0 ^ \"1.0\"");
assert_bitxor(1.0, "1.0");

trace("//-1.0 ^ \"1.0\"");
assert_bitxor(-1.0, "1.0");

trace("//0xFF1306 ^ \"1.0\"");
assert_bitxor(0xFF1306, "1.0");

trace("//new Object() ^ \"1.0\"");
assert_bitxor({}, "1.0");

trace("//\"0.0\" ^ \"1.0\"");
assert_bitxor("0.0", "1.0");

trace("//\"NaN\" ^ \"1.0\"");
assert_bitxor("NaN", "1.0");

trace("//\"-0.0\" ^ \"1.0\"");
assert_bitxor("-0.0", "1.0");

trace("//\"Infinity\" ^ \"1.0\"");
assert_bitxor("Infinity", "1.0");

trace("//\"1.0\" ^ \"1.0\"");
assert_bitxor("1.0", "1.0");

trace("//\"-1.0\" ^ \"1.0\"");
assert_bitxor("-1.0", "1.0");

trace("//\"0xFF1306\" ^ \"1.0\"");
assert_bitxor("0xFF1306", "1.0");

trace("//true ^ \"-1.0\"");
assert_bitxor(true, "-1.0");

trace("//false ^ \"-1.0\"");
assert_bitxor(false, "-1.0");

trace("//null ^ \"-1.0\"");
assert_bitxor(null, "-1.0");

trace("//undefined ^ \"-1.0\"");
assert_bitxor(undefined, "-1.0");

trace("//\"\" ^ \"-1.0\"");
assert_bitxor("", "-1.0");

trace("//\"str\" ^ \"-1.0\"");
assert_bitxor("str", "-1.0");

trace("//\"true\" ^ \"-1.0\"");
assert_bitxor("true", "-1.0");

trace("//\"false\" ^ \"-1.0\"");
assert_bitxor("false", "-1.0");

trace("//0.0 ^ \"-1.0\"");
assert_bitxor(0.0, "-1.0");

trace("//NaN ^ \"-1.0\"");
assert_bitxor(NaN, "-1.0");

trace("//-0.0 ^ \"-1.0\"");
assert_bitxor(-0.0, "-1.0");

trace("//Infinity ^ \"-1.0\"");
assert_bitxor(Infinity, "-1.0");

trace("//1.0 ^ \"-1.0\"");
assert_bitxor(1.0, "-1.0");

trace("//-1.0 ^ \"-1.0\"");
assert_bitxor(-1.0, "-1.0");

trace("//0xFF1306 ^ \"-1.0\"");
assert_bitxor(0xFF1306, "-1.0");

trace("//new Object() ^ \"-1.0\"");
assert_bitxor({}, "-1.0");

trace("//\"0.0\" ^ \"-1.0\"");
assert_bitxor("0.0", "-1.0");

trace("//\"NaN\" ^ \"-1.0\"");
assert_bitxor("NaN", "-1.0");

trace("//\"-0.0\" ^ \"-1.0\"");
assert_bitxor("-0.0", "-1.0");

trace("//\"Infinity\" ^ \"-1.0\"");
assert_bitxor("Infinity", "-1.0");

trace("//\"1.0\" ^ \"-1.0\"");
assert_bitxor("1.0", "-1.0");

trace("//\"-1.0\" ^ \"-1.0\"");
assert_bitxor("-1.0", "-1.0");

trace("//\"0xFF1306\" ^ \"-1.0\"");
assert_bitxor("0xFF1306", "-1.0");

trace("//true ^ \"0xFF1306\"");
assert_bitxor(true, "0xFF1306");

trace("//false ^ \"0xFF1306\"");
assert_bitxor(false, "0xFF1306");

trace("//null ^ \"0xFF1306\"");
assert_bitxor(null, "0xFF1306");

trace("//undefined ^ \"0xFF1306\"");
assert_bitxor(undefined, "0xFF1306");

trace("//\"\" ^ \"0xFF1306\"");
assert_bitxor("", "0xFF1306");

trace("//\"str\" ^ \"0xFF1306\"");
assert_bitxor("str", "0xFF1306");

trace("//\"true\" ^ \"0xFF1306\"");
assert_bitxor("true", "0xFF1306");

trace("//\"false\" ^ \"0xFF1306\"");
assert_bitxor("false", "0xFF1306");

trace("//0.0 ^ \"0xFF1306\"");
assert_bitxor(0.0, "0xFF1306");

trace("//NaN ^ \"0xFF1306\"");
assert_bitxor(NaN, "0xFF1306");

trace("//-0.0 ^ \"0xFF1306\"");
assert_bitxor(-0.0, "0xFF1306");

trace("//Infinity ^ \"0xFF1306\"");
assert_bitxor(Infinity, "0xFF1306");

trace("//1.0 ^ \"0xFF1306\"");
assert_bitxor(1.0, "0xFF1306");

trace("//-1.0 ^ \"0xFF1306\"");
assert_bitxor(-1.0, "0xFF1306");

trace("//0xFF1306 ^ \"0xFF1306\"");
assert_bitxor(0xFF1306, "0xFF1306");

trace("//new Object() ^ \"0xFF1306\"");
assert_bitxor({}, "0xFF1306");

trace("//\"0.0\" ^ \"0xFF1306\"");
assert_bitxor("0.0", "0xFF1306");

trace("//\"NaN\" ^ \"0xFF1306\"");
assert_bitxor("NaN", "0xFF1306");

trace("//\"-0.0\" ^ \"0xFF1306\"");
assert_bitxor("-0.0", "0xFF1306");

trace("//\"Infinity\" ^ \"0xFF1306\"");
assert_bitxor("Infinity", "0xFF1306");

trace("//\"1.0\" ^ \"0xFF1306\"");
assert_bitxor("1.0", "0xFF1306");

trace("//\"-1.0\" ^ \"0xFF1306\"");
assert_bitxor("-1.0", "0xFF1306");

trace("//\"0xFF1306\" ^ \"0xFF1306\"");
assert_bitxor("0xFF1306", "0xFF1306");