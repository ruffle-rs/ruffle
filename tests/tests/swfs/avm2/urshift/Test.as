package {
	public class Test {
	}
}

function assert_urshift(val1, val2) {
	trace(val1 >>> val2);
}

trace("//true >>> true");
assert_urshift(true, true);

trace("//false >>> true");
assert_urshift(false, true);

trace("//null >>> true");
assert_urshift(null, true);

trace("//undefined >>> true");
assert_urshift(undefined, true);

trace("//\"\" >>> true");
assert_urshift("", true);

trace("//\"str\" >>> true");
assert_urshift("str", true);

trace("//\"true\" >>> true");
assert_urshift("true", true);

trace("//\"false\" >>> true");
assert_urshift("false", true);

trace("//0.0 >>> true");
assert_urshift(0.0, true);

trace("//NaN >>> true");
assert_urshift(NaN, true);

trace("//-0.0 >>> true");
assert_urshift(-0.0, true);

trace("//Infinity >>> true");
assert_urshift(Infinity, true);

trace("//1.0 >>> true");
assert_urshift(1.0, true);

trace("//-1.0 >>> true");
assert_urshift(-1.0, true);

trace("//0xFF1306 >>> true");
assert_urshift(0xFF1306, true);

trace("//new Object() >>> true");
assert_urshift({}, true);

trace("//\"0.0\" >>> true");
assert_urshift("0.0", true);

trace("//\"NaN\" >>> true");
assert_urshift("NaN", true);

trace("//\"-0.0\" >>> true");
assert_urshift("-0.0", true);

trace("//\"Infinity\" >>> true");
assert_urshift("Infinity", true);

trace("//\"1.0\" >>> true");
assert_urshift("1.0", true);

trace("//\"-1.0\" >>> true");
assert_urshift("-1.0", true);

trace("//\"0xFF1306\" >>> true");
assert_urshift("0xFF1306", true);

trace("//true >>> false");
assert_urshift(true, false);

trace("//false >>> false");
assert_urshift(false, false);

trace("//null >>> false");
assert_urshift(null, false);

trace("//undefined >>> false");
assert_urshift(undefined, false);

trace("//\"\" >>> false");
assert_urshift("", false);

trace("//\"str\" >>> false");
assert_urshift("str", false);

trace("//\"true\" >>> false");
assert_urshift("true", false);

trace("//\"false\" >>> false");
assert_urshift("false", false);

trace("//0.0 >>> false");
assert_urshift(0.0, false);

trace("//NaN >>> false");
assert_urshift(NaN, false);

trace("//-0.0 >>> false");
assert_urshift(-0.0, false);

trace("//Infinity >>> false");
assert_urshift(Infinity, false);

trace("//1.0 >>> false");
assert_urshift(1.0, false);

trace("//-1.0 >>> false");
assert_urshift(-1.0, false);

trace("//0xFF1306 >>> false");
assert_urshift(0xFF1306, false);

trace("//new Object() >>> false");
assert_urshift({}, false);

trace("//\"0.0\" >>> false");
assert_urshift("0.0", false);

trace("//\"NaN\" >>> false");
assert_urshift("NaN", false);

trace("//\"-0.0\" >>> false");
assert_urshift("-0.0", false);

trace("//\"Infinity\" >>> false");
assert_urshift("Infinity", false);

trace("//\"1.0\" >>> false");
assert_urshift("1.0", false);

trace("//\"-1.0\" >>> false");
assert_urshift("-1.0", false);

trace("//\"0xFF1306\" >>> false");
assert_urshift("0xFF1306", false);
trace("//true >>> null");
assert_urshift(true, null);

trace("//false >>> null");
assert_urshift(false, null);

trace("//null >>> null");
assert_urshift(null, null);

trace("//undefined >>> null");
assert_urshift(undefined, null);

trace("//\"\" >>> null");
assert_urshift("", null);

trace("//\"str\" >>> null");
assert_urshift("str", null);

trace("//\"true\" >>> null");
assert_urshift("true", null);

trace("//\"false\" >>> null");
assert_urshift("false", null);

trace("//0.0 >>> null");
assert_urshift(0.0, null);

trace("//NaN >>> null");
assert_urshift(NaN, null);

trace("//-0.0 >>> null");
assert_urshift(-0.0, null);

trace("//Infinity >>> null");
assert_urshift(Infinity, null);

trace("//1.0 >>> null");
assert_urshift(1.0, null);

trace("//-1.0 >>> null");
assert_urshift(-1.0, null);

trace("//0xFF1306 >>> null");
assert_urshift(0xFF1306, null);

trace("//new Object() >>> null");
assert_urshift({}, null);

trace("//\"0.0\" >>> null");
assert_urshift("0.0", null);

trace("//\"NaN\" >>> null");
assert_urshift("NaN", null);

trace("//\"-0.0\" >>> null");
assert_urshift("-0.0", null);

trace("//\"Infinity\" >>> null");
assert_urshift("Infinity", null);

trace("//\"1.0\" >>> null");
assert_urshift("1.0", null);

trace("//\"-1.0\" >>> null");
assert_urshift("-1.0", null);

trace("//\"0xFF1306\" >>> null");
assert_urshift("0xFF1306", null);

trace("//true >>> undefined");
assert_urshift(true, undefined);

trace("//false >>> undefined");
assert_urshift(false, undefined);

trace("//null >>> undefined");
assert_urshift(null, undefined);

trace("//undefined >>> undefined");
assert_urshift(undefined, undefined);

trace("//\"\" >>> undefined");
assert_urshift("", undefined);

trace("//\"str\" >>> undefined");
assert_urshift("str", undefined);

trace("//\"true\" >>> undefined");
assert_urshift("true", undefined);

trace("//\"false\" >>> undefined");
assert_urshift("false", undefined);

trace("//0.0 >>> undefined");
assert_urshift(0.0, undefined);

trace("//NaN >>> undefined");
assert_urshift(NaN, undefined);

trace("//-0.0 >>> undefined");
assert_urshift(-0.0, undefined);

trace("//Infinity >>> undefined");
assert_urshift(Infinity, undefined);

trace("//1.0 >>> undefined");
assert_urshift(1.0, undefined);

trace("//-1.0 >>> undefined");
assert_urshift(-1.0, undefined);

trace("//0xFF1306 >>> undefined");
assert_urshift(0xFF1306, undefined);

trace("//new Object() >>> undefined");
assert_urshift({}, undefined);

trace("//\"0.0\" >>> undefined");
assert_urshift("0.0", undefined);

trace("//\"NaN\" >>> undefined");
assert_urshift("NaN", undefined);

trace("//\"-0.0\" >>> undefined");
assert_urshift("-0.0", undefined);

trace("//\"Infinity\" >>> undefined");
assert_urshift("Infinity", undefined);

trace("//\"1.0\" >>> undefined");
assert_urshift("1.0", undefined);

trace("//\"-1.0\" >>> undefined");
assert_urshift("-1.0", undefined);

trace("//\"0xFF1306\" >>> undefined");
assert_urshift("0xFF1306", undefined);

trace("//true >>> \"\"");
assert_urshift(true, "");

trace("//false >>> \"\"");
assert_urshift(false, "");

trace("//null >>> \"\"");
assert_urshift(null, "");

trace("//undefined >>> \"\"");
assert_urshift(undefined, "");

trace("//\"\" >>> \"\"");
assert_urshift("", "");

trace("//\"str\" >>> \"\"");
assert_urshift("str", "");

trace("//\"true\" >>> \"\"");
assert_urshift("true", "");

trace("//\"false\" >>> \"\"");
assert_urshift("false", "");

trace("//0.0 >>> \"\"");
assert_urshift(0.0, "");

trace("//NaN >>> \"\"");
assert_urshift(NaN, "");

trace("//-0.0 >>> \"\"");
assert_urshift(-0.0, "");

trace("//Infinity >>> \"\"");
assert_urshift(Infinity, "");

trace("//1.0 >>> \"\"");
assert_urshift(1.0, "");

trace("//-1.0 >>> \"\"");
assert_urshift(-1.0, "");

trace("//0xFF1306 >>> \"\"");
assert_urshift(0xFF1306, "");

trace("//new Object() >>> \"\"");
assert_urshift({}, "");

trace("//\"0.0\" >>> \"\"");
assert_urshift("0.0", "");

trace("//\"NaN\" >>> \"\"");
assert_urshift("NaN", "");

trace("//\"-0.0\" >>> \"\"");
assert_urshift("-0.0", "");

trace("//\"Infinity\" >>> \"\"");
assert_urshift("Infinity", "");

trace("//\"1.0\" >>> \"\"");
assert_urshift("1.0", "");

trace("//\"-1.0\" >>> \"\"");
assert_urshift("-1.0", "");

trace("//\"0xFF1306\" >>> \"\"");
assert_urshift("0xFF1306", "");

trace("//true >>> \"str\"");
assert_urshift(true, "str");

trace("//false >>> \"str\"");
assert_urshift(false, "str");

trace("//null >>> \"str\"");
assert_urshift(null, "str");

trace("//undefined >>> \"str\"");
assert_urshift(undefined, "str");

trace("//\"\" >>> \"str\"");
assert_urshift("", "str");

trace("//\"str\" >>> \"str\"");
assert_urshift("str", "str");

trace("//\"true\" >>> \"str\"");
assert_urshift("true", "str");

trace("//\"false\" >>> \"str\"");
assert_urshift("false", "str");

trace("//0.0 >>> \"str\"");
assert_urshift(0.0, "str");

trace("//NaN >>> \"str\"");
assert_urshift(NaN, "str");

trace("//-0.0 >>> \"str\"");
assert_urshift(-0.0, "str");

trace("//Infinity >>> \"str\"");
assert_urshift(Infinity, "str");

trace("//1.0 >>> \"str\"");
assert_urshift(1.0, "str");

trace("//-1.0 >>> \"str\"");
assert_urshift(-1.0, "str");

trace("//0xFF1306 >>> \"str\"");
assert_urshift(0xFF1306, "str");

trace("//new Object() >>> \"str\"");
assert_urshift({}, "str");

trace("//\"0.0\" >>> \"str\"");
assert_urshift("0.0", "str");

trace("//\"NaN\" >>> \"str\"");
assert_urshift("NaN", "str");

trace("//\"-0.0\" >>> \"str\"");
assert_urshift("-0.0", "str");

trace("//\"Infinity\" >>> \"str\"");
assert_urshift("Infinity", "str");

trace("//\"1.0\" >>> \"str\"");
assert_urshift("1.0", "str");

trace("//\"-1.0\" >>> \"str\"");
assert_urshift("-1.0", "str");

trace("//\"0xFF1306\" >>> \"str\"");
assert_urshift("0xFF1306", "str");

trace("//true >>> \"true\"");
assert_urshift(true, "true");

trace("//false >>> \"true\"");
assert_urshift(false, "true");

trace("//null >>> \"true\"");
assert_urshift(null, "true");

trace("//undefined >>> \"true\"");
assert_urshift(undefined, "true");

trace("//\"\" >>> \"true\"");
assert_urshift("", "true");

trace("//\"str\" >>> \"true\"");
assert_urshift("str", "true");

trace("//\"true\" >>> \"true\"");
assert_urshift("true", "true");

trace("//\"false\" >>> \"true\"");
assert_urshift("false", "true");

trace("//0.0 >>> \"true\"");
assert_urshift(0.0, "true");

trace("//NaN >>> \"true\"");
assert_urshift(NaN, "true");

trace("//-0.0 >>> \"true\"");
assert_urshift(-0.0, "true");

trace("//Infinity >>> \"true\"");
assert_urshift(Infinity, "true");

trace("//1.0 >>> \"true\"");
assert_urshift(1.0, "true");

trace("//-1.0 >>> \"true\"");
assert_urshift(-1.0, "true");

trace("//0xFF1306 >>> \"true\"");
assert_urshift(0xFF1306, "true");

trace("//new Object() >>> \"true\"");
assert_urshift({}, "true");

trace("//\"0.0\" >>> \"true\"");
assert_urshift("0.0", "true");

trace("//\"NaN\" >>> \"true\"");
assert_urshift("NaN", "true");

trace("//\"-0.0\" >>> \"true\"");
assert_urshift("-0.0", "true");

trace("//\"Infinity\" >>> \"true\"");
assert_urshift("Infinity", "true");

trace("//\"1.0\" >>> \"true\"");
assert_urshift("1.0", "true");

trace("//\"-1.0\" >>> \"true\"");
assert_urshift("-1.0", "true");

trace("//\"0xFF1306\" >>> \"true\"");
assert_urshift("0xFF1306", "true");

trace("//true >>> \"false\"");
assert_urshift(true, "false");

trace("//false >>> \"false\"");
assert_urshift(false, "false");

trace("//null >>> \"false\"");
assert_urshift(null, "false");

trace("//undefined >>> \"false\"");
assert_urshift(undefined, "false");

trace("//\"\" >>> \"false\"");
assert_urshift("", "false");

trace("//\"str\" >>> \"false\"");
assert_urshift("str", "false");

trace("//\"true\" >>> \"false\"");
assert_urshift("true", "false");

trace("//\"false\" >>> \"false\"");
assert_urshift("false", "false");

trace("//0.0 >>> \"false\"");
assert_urshift(0.0, "false");

trace("//NaN >>> \"false\"");
assert_urshift(NaN, "false");

trace("//-0.0 >>> \"false\"");
assert_urshift(-0.0, "false");

trace("//Infinity >>> \"false\"");
assert_urshift(Infinity, "false");

trace("//1.0 >>> \"false\"");
assert_urshift(1.0, "false");

trace("//-1.0 >>> \"false\"");
assert_urshift(-1.0, "false");

trace("//0xFF1306 >>> \"false\"");
assert_urshift(0xFF1306, "false");

trace("//new Object() >>> \"false\"");
assert_urshift({}, "false");

trace("//\"0.0\" >>> \"false\"");
assert_urshift("0.0", "false");

trace("//\"NaN\" >>> \"false\"");
assert_urshift("NaN", "false");

trace("//\"-0.0\" >>> \"false\"");
assert_urshift("-0.0", "false");

trace("//\"Infinity\" >>> \"false\"");
assert_urshift("Infinity", "false");

trace("//\"1.0\" >>> \"false\"");
assert_urshift("1.0", "false");

trace("//\"-1.0\" >>> \"false\"");
assert_urshift("-1.0", "false");

trace("//\"0xFF1306\" >>> \"false\"");
assert_urshift("0xFF1306", "false");

trace("//true >>> 0.0");
assert_urshift(true, 0.0);

trace("//false >>> 0.0");
assert_urshift(false, 0.0);

trace("//null >>> 0.0");
assert_urshift(null, 0.0);

trace("//undefined >>> 0.0");
assert_urshift(undefined, 0.0);

trace("//\"\" >>> 0.0");
assert_urshift("", 0.0);

trace("//\"str\" >>> 0.0");
assert_urshift("str", 0.0);

trace("//\"true\" >>> 0.0");
assert_urshift("true", 0.0);

trace("//\"false\" >>> 0.0");
assert_urshift("false", 0.0);

trace("//0.0 >>> 0.0");
assert_urshift(0.0, 0.0);

trace("//NaN >>> 0.0");
assert_urshift(NaN, 0.0);

trace("//-0.0 >>> 0.0");
assert_urshift(-0.0, 0.0);

trace("//Infinity >>> 0.0");
assert_urshift(Infinity, 0.0);

trace("//1.0 >>> 0.0");
assert_urshift(1.0, 0.0);

trace("//-1.0 >>> 0.0");
assert_urshift(-1.0, 0.0);

trace("//0xFF1306 >>> 0.0");
assert_urshift(0xFF1306, 0.0);

trace("//new Object() >>> 0.0");
assert_urshift({}, 0.0);

trace("//\"0.0\" >>> 0.0");
assert_urshift("0.0", 0.0);

trace("//\"NaN\" >>> 0.0");
assert_urshift("NaN", 0.0);

trace("//\"-0.0\" >>> 0.0");
assert_urshift("-0.0", 0.0);

trace("//\"Infinity\" >>> 0.0");
assert_urshift("Infinity", 0.0);

trace("//\"1.0\" >>> 0.0");
assert_urshift("1.0", 0.0);

trace("//\"-1.0\" >>> 0.0");
assert_urshift("-1.0", 0.0);

trace("//\"0xFF1306\" >>> 0.0");
assert_urshift("0xFF1306", 0.0);

trace("//true >>> NaN");
assert_urshift(true, NaN);

trace("//false >>> NaN");
assert_urshift(false, NaN);

trace("//null >>> NaN");
assert_urshift(null, NaN);

trace("//undefined >>> NaN");
assert_urshift(undefined, NaN);

trace("//\"\" >>> NaN");
assert_urshift("", NaN);

trace("//\"str\" >>> NaN");
assert_urshift("str", NaN);

trace("//\"true\" >>> NaN");
assert_urshift("true", NaN);

trace("//\"false\" >>> NaN");
assert_urshift("false", NaN);

trace("//0.0 >>> NaN");
assert_urshift(0.0, NaN);

trace("//NaN >>> NaN");
assert_urshift(NaN, NaN);

trace("//-0.0 >>> NaN");
assert_urshift(-0.0, NaN);

trace("//Infinity >>> NaN");
assert_urshift(Infinity, NaN);

trace("//1.0 >>> NaN");
assert_urshift(1.0, NaN);

trace("//-1.0 >>> NaN");
assert_urshift(-1.0, NaN);

trace("//0xFF1306 >>> NaN");
assert_urshift(0xFF1306, NaN);

trace("//new Object() >>> NaN");
assert_urshift({}, NaN);

trace("//\"0.0\" >>> NaN");
assert_urshift("0.0", NaN);

trace("//\"NaN\" >>> NaN");
assert_urshift("NaN", NaN);

trace("//\"-0.0\" >>> NaN");
assert_urshift("-0.0", NaN);

trace("//\"Infinity\" >>> NaN");
assert_urshift("Infinity", NaN);

trace("//\"1.0\" >>> NaN");
assert_urshift("1.0", NaN);

trace("//\"-1.0\" >>> NaN");
assert_urshift("-1.0", NaN);

trace("//\"0xFF1306\" >>> NaN");
assert_urshift("0xFF1306", NaN);

trace("//true >>> -0.0");
assert_urshift(true, -0.0);

trace("//false >>> -0.0");
assert_urshift(false, -0.0);

trace("//null >>> -0.0");
assert_urshift(null, -0.0);

trace("//undefined >>> -0.0");
assert_urshift(undefined, -0.0);

trace("//\"\" >>> -0.0");
assert_urshift("", -0.0);

trace("//\"str\" >>> -0.0");
assert_urshift("str", -0.0);

trace("//\"true\" >>> -0.0");
assert_urshift("true", -0.0);

trace("//\"false\" >>> -0.0");
assert_urshift("false", -0.0);

trace("//0.0 >>> -0.0");
assert_urshift(0.0, -0.0);

trace("//NaN >>> -0.0");
assert_urshift(NaN, -0.0);

trace("//-0.0 >>> -0.0");
assert_urshift(-0.0, -0.0);

trace("//Infinity >>> -0.0");
assert_urshift(Infinity, -0.0);

trace("//1.0 >>> -0.0");
assert_urshift(1.0, -0.0);

trace("//-1.0 >>> -0.0");
assert_urshift(-1.0, -0.0);

trace("//0xFF1306 >>> -0.0");
assert_urshift(0xFF1306, -0.0);

trace("//new Object() >>> -0.0");
assert_urshift({}, -0.0);

trace("//\"0.0\" >>> -0.0");
assert_urshift("0.0", -0.0);

trace("//\"NaN\" >>> -0.0");
assert_urshift("NaN", -0.0);

trace("//\"-0.0\" >>> -0.0");
assert_urshift("-0.0", -0.0);

trace("//\"Infinity\" >>> -0.0");
assert_urshift("Infinity", -0.0);

trace("//\"1.0\" >>> -0.0");
assert_urshift("1.0", -0.0);

trace("//\"-1.0\" >>> -0.0");
assert_urshift("-1.0", -0.0);

trace("//\"0xFF1306\" >>> -0.0");
assert_urshift("0xFF1306", -0.0);

trace("//true >>> Infinity");
assert_urshift(true, Infinity);

trace("//false >>> Infinity");
assert_urshift(false, Infinity);

trace("//null >>> Infinity");
assert_urshift(null, Infinity);

trace("//undefined >>> Infinity");
assert_urshift(undefined, Infinity);

trace("//\"\" >>> Infinity");
assert_urshift("", Infinity);

trace("//\"str\" >>> Infinity");
assert_urshift("str", Infinity);

trace("//\"true\" >>> Infinity");
assert_urshift("true", Infinity);

trace("//\"false\" >>> Infinity");
assert_urshift("false", Infinity);

trace("//0.0 >>> Infinity");
assert_urshift(0.0, Infinity);

trace("//NaN >>> Infinity");
assert_urshift(NaN, Infinity);

trace("//-0.0 >>> Infinity");
assert_urshift(-0.0, Infinity);

trace("//Infinity >>> Infinity");
assert_urshift(Infinity, Infinity);

trace("//1.0 >>> Infinity");
assert_urshift(1.0, Infinity);

trace("//-1.0 >>> Infinity");
assert_urshift(-1.0, Infinity);

trace("//0xFF1306 >>> Infinity");
assert_urshift(0xFF1306, Infinity);

trace("//new Object() >>> Infinity");
assert_urshift({}, Infinity);

trace("//\"0.0\" >>> Infinity");
assert_urshift("0.0", Infinity);

trace("//\"NaN\" >>> Infinity");
assert_urshift("NaN", Infinity);

trace("//\"-0.0\" >>> Infinity");
assert_urshift("-0.0", Infinity);

trace("//\"Infinity\" >>> Infinity");
assert_urshift("Infinity", Infinity);

trace("//\"1.0\" >>> Infinity");
assert_urshift("1.0", Infinity);

trace("//\"-1.0\" >>> Infinity");
assert_urshift("-1.0", Infinity);

trace("//\"0xFF1306\" >>> Infinity");
assert_urshift("0xFF1306", Infinity);

trace("//true >>> 1.0");
assert_urshift(true, 1.0);

trace("//false >>> 1.0");
assert_urshift(false, 1.0);

trace("//null >>> 1.0");
assert_urshift(null, 1.0);

trace("//undefined >>> 1.0");
assert_urshift(undefined, 1.0);

trace("//\"\" >>> 1.0");
assert_urshift("", 1.0);

trace("//\"str\" >>> 1.0");
assert_urshift("str", 1.0);

trace("//\"true\" >>> 1.0");
assert_urshift("true", 1.0);

trace("//\"false\" >>> 1.0");
assert_urshift("false", 1.0);

trace("//0.0 >>> 1.0");
assert_urshift(0.0, 1.0);

trace("//NaN >>> 1.0");
assert_urshift(NaN, 1.0);

trace("//-0.0 >>> 1.0");
assert_urshift(-0.0, 1.0);

trace("//Infinity >>> 1.0");
assert_urshift(Infinity, 1.0);

trace("//1.0 >>> 1.0");
assert_urshift(1.0, 1.0);

trace("//-1.0 >>> 1.0");
assert_urshift(-1.0, 1.0);

trace("//0xFF1306 >>> 1.0");
assert_urshift(0xFF1306, 1.0);

trace("//new Object() >>> 1.0");
assert_urshift({}, 1.0);

trace("//\"0.0\" >>> 1.0");
assert_urshift("0.0", 1.0);

trace("//\"NaN\" >>> 1.0");
assert_urshift("NaN", 1.0);

trace("//\"-0.0\" >>> 1.0");
assert_urshift("-0.0", 1.0);

trace("//\"Infinity\" >>> 1.0");
assert_urshift("Infinity", 1.0);

trace("//\"1.0\" >>> 1.0");
assert_urshift("1.0", 1.0);

trace("//\"-1.0\" >>> 1.0");
assert_urshift("-1.0", 1.0);

trace("//\"0xFF1306\" >>> 1.0");
assert_urshift("0xFF1306", 1.0);

trace("//true >>> -1.0");
assert_urshift(true, -1.0);

trace("//false >>> -1.0");
assert_urshift(false, -1.0);

trace("//null >>> -1.0");
assert_urshift(null, -1.0);

trace("//undefined >>> -1.0");
assert_urshift(undefined, -1.0);

trace("//\"\" >>> -1.0");
assert_urshift("", -1.0);

trace("//\"str\" >>> -1.0");
assert_urshift("str", -1.0);

trace("//\"true\" >>> -1.0");
assert_urshift("true", -1.0);

trace("//\"false\" >>> -1.0");
assert_urshift("false", -1.0);

trace("//0.0 >>> -1.0");
assert_urshift(0.0, -1.0);

trace("//NaN >>> -1.0");
assert_urshift(NaN, -1.0);

trace("//-0.0 >>> -1.0");
assert_urshift(-0.0, -1.0);

trace("//Infinity >>> -1.0");
assert_urshift(Infinity, -1.0);

trace("//1.0 >>> -1.0");
assert_urshift(1.0, -1.0);

trace("//-1.0 >>> -1.0");
assert_urshift(-1.0, -1.0);

trace("//0xFF1306 >>> -1.0");
assert_urshift(0xFF1306, -1.0);

trace("//new Object() >>> -1.0");
assert_urshift({}, -1.0);

trace("//\"0.0\" >>> -1.0");
assert_urshift("0.0", -1.0);

trace("//\"NaN\" >>> -1.0");
assert_urshift("NaN", -1.0);

trace("//\"-0.0\" >>> -1.0");
assert_urshift("-0.0", -1.0);

trace("//\"Infinity\" >>> -1.0");
assert_urshift("Infinity", -1.0);

trace("//\"1.0\" >>> -1.0");
assert_urshift("1.0", -1.0);

trace("//\"-1.0\" >>> -1.0");
assert_urshift("-1.0", -1.0);

trace("//\"0xFF1306\" >>> -1.0");
assert_urshift("0xFF1306", -1.0);

trace("//true >>> 0xFF1306");
assert_urshift(true, 0xFF1306);

trace("//false >>> 0xFF1306");
assert_urshift(false, 0xFF1306);

trace("//null >>> 0xFF1306");
assert_urshift(null, 0xFF1306);

trace("//undefined >>> 0xFF1306");
assert_urshift(undefined, 0xFF1306);

trace("//\"\" >>> 0xFF1306");
assert_urshift("", 0xFF1306);

trace("//\"str\" >>> 0xFF1306");
assert_urshift("str", 0xFF1306);

trace("//\"true\" >>> 0xFF1306");
assert_urshift("true", 0xFF1306);

trace("//\"false\" >>> 0xFF1306");
assert_urshift("false", 0xFF1306);

trace("//0.0 >>> 0xFF1306");
assert_urshift(0.0, 0xFF1306);

trace("//NaN >>> 0xFF1306");
assert_urshift(NaN, 0xFF1306);

trace("//-0.0 >>> 0xFF1306");
assert_urshift(-0.0, 0xFF1306);

trace("//Infinity >>> 0xFF1306");
assert_urshift(Infinity, 0xFF1306);

trace("//1.0 >>> 0xFF1306");
assert_urshift(1.0, 0xFF1306);

trace("//-1.0 >>> 0xFF1306");
assert_urshift(-1.0, 0xFF1306);

trace("//0xFF1306 >>> 0xFF1306");
assert_urshift(0xFF1306, 0xFF1306);

trace("//new Object() >>> 0xFF1306");
assert_urshift({}, 0xFF1306);

trace("//\"0.0\" >>> 0xFF1306");
assert_urshift("0.0", 0xFF1306);

trace("//\"NaN\" >>> 0xFF1306");
assert_urshift("NaN", 0xFF1306);

trace("//\"-0.0\" >>> 0xFF1306");
assert_urshift("-0.0", 0xFF1306);

trace("//\"Infinity\" >>> 0xFF1306");
assert_urshift("Infinity", 0xFF1306);

trace("//\"1.0\" >>> 0xFF1306");
assert_urshift("1.0", 0xFF1306);

trace("//\"-1.0\" >>> 0xFF1306");
assert_urshift("-1.0", 0xFF1306);

trace("//\"0xFF1306\" >>> 0xFF1306");
assert_urshift("0xFF1306", 0xFF1306);

trace("//true >>> new Object()");
assert_urshift(true, {});

trace("//false >>> new Object()");
assert_urshift(false, {});

trace("//null >>> new Object()");
assert_urshift(null, {});

trace("//undefined >>> new Object()");
assert_urshift(undefined, {});

trace("//\"\" >>> new Object()");
assert_urshift("", {});

trace("//\"str\" >>> new Object()");
assert_urshift("str", {});

trace("//\"true\" >>> new Object()");
assert_urshift("true", {});

trace("//\"false\" >>> new Object()");
assert_urshift("false", {});

trace("//0.0 >>> new Object()");
assert_urshift(0.0, {});

trace("//NaN >>> new Object()");
assert_urshift(NaN, {});

trace("//-0.0 >>> new Object()");
assert_urshift(-0.0, {});

trace("//Infinity >>> new Object()");
assert_urshift(Infinity, {});

trace("//1.0 >>> new Object()");
assert_urshift(1.0, {});

trace("//-1.0 >>> new Object()");
assert_urshift(-1.0, {});

trace("//0xFF1306 >>> new Object()");
assert_urshift(0xFF1306, {});

trace("//new Object() >>> new Object()");
assert_urshift({}, {});

trace("//\"0.0\" >>> new Object()");
assert_urshift("0.0", {});

trace("//\"NaN\" >>> new Object()");
assert_urshift("NaN", {});

trace("//\"-0.0\" >>> new Object()");
assert_urshift("-0.0", {});

trace("//\"Infinity\" >>> new Object()");
assert_urshift("Infinity", {});

trace("//\"1.0\" >>> new Object()");
assert_urshift("1.0", {});

trace("//\"-1.0\" >>> new Object()");
assert_urshift("-1.0", {});

trace("//\"0xFF1306\" >>> new Object()");
assert_urshift("0xFF1306", {});

trace("//true >>> \"0.0\"");
assert_urshift(true, "0.0");

trace("//false >>> \"0.0\"");
assert_urshift(false, "0.0");

trace("//null >>> \"0.0\"");
assert_urshift(null, "0.0");

trace("//undefined >>> \"0.0\"");
assert_urshift(undefined, "0.0");

trace("//\"\" >>> \"0.0\"");
assert_urshift("", "0.0");

trace("//\"str\" >>> \"0.0\"");
assert_urshift("str", "0.0");

trace("//\"true\" >>> \"0.0\"");
assert_urshift("true", "0.0");

trace("//\"false\" >>> \"0.0\"");
assert_urshift("false", "0.0");

trace("//0.0 >>> \"0.0\"");
assert_urshift(0.0, "0.0");

trace("//NaN >>> \"0.0\"");
assert_urshift(NaN, "0.0");

trace("//-0.0 >>> \"0.0\"");
assert_urshift(-0.0, "0.0");

trace("//Infinity >>> \"0.0\"");
assert_urshift(Infinity, "0.0");

trace("//1.0 >>> \"0.0\"");
assert_urshift(1.0, "0.0");

trace("//-1.0 >>> \"0.0\"");
assert_urshift(-1.0, "0.0");

trace("//0xFF1306 >>> \"0.0\"");
assert_urshift(0xFF1306, "0.0");

trace("//new Object() >>> \"0.0\"");
assert_urshift({}, "0.0");

trace("//\"0.0\" >>> \"0.0\"");
assert_urshift("0.0", "0.0");

trace("//\"NaN\" >>> \"0.0\"");
assert_urshift("NaN", "0.0");

trace("//\"-0.0\" >>> \"0.0\"");
assert_urshift("-0.0", "0.0");

trace("//\"Infinity\" >>> \"0.0\"");
assert_urshift("Infinity", "0.0");

trace("//\"1.0\" >>> \"0.0\"");
assert_urshift("1.0", "0.0");

trace("//\"-1.0\" >>> \"0.0\"");
assert_urshift("-1.0", "0.0");

trace("//\"0xFF1306\" >>> \"0.0\"");
assert_urshift("0xFF1306", "0.0");

trace("//true >>> \"NaN\"");
assert_urshift(true, "NaN");

trace("//false >>> \"NaN\"");
assert_urshift(false, "NaN");

trace("//null >>> \"NaN\"");
assert_urshift(null, "NaN");

trace("//undefined >>> \"NaN\"");
assert_urshift(undefined, "NaN");

trace("//\"\" >>> \"NaN\"");
assert_urshift("", "NaN");

trace("//\"str\" >>> \"NaN\"");
assert_urshift("str", "NaN");

trace("//\"true\" >>> \"NaN\"");
assert_urshift("true", "NaN");

trace("//\"false\" >>> \"NaN\"");
assert_urshift("false", "NaN");

trace("//0.0 >>> \"NaN\"");
assert_urshift(0.0, "NaN");

trace("//NaN >>> \"NaN\"");
assert_urshift(NaN, "NaN");

trace("//-0.0 >>> \"NaN\"");
assert_urshift(-0.0, "NaN");

trace("//Infinity >>> \"NaN\"");
assert_urshift(Infinity, "NaN");

trace("//1.0 >>> \"NaN\"");
assert_urshift(1.0, "NaN");

trace("//-1.0 >>> \"NaN\"");
assert_urshift(-1.0, "NaN");

trace("//0xFF1306 >>> \"NaN\"");
assert_urshift(0xFF1306, "NaN");

trace("//new Object() >>> \"NaN\"");
assert_urshift({}, "NaN");

trace("//\"0.0\" >>> \"NaN\"");
assert_urshift("0.0", "NaN");

trace("//\"NaN\" >>> \"NaN\"");
assert_urshift("NaN", "NaN");

trace("//\"-0.0\" >>> \"NaN\"");
assert_urshift("-0.0", "NaN");

trace("//\"Infinity\" >>> \"NaN\"");
assert_urshift("Infinity", "NaN");

trace("//\"1.0\" >>> \"NaN\"");
assert_urshift("1.0", "NaN");

trace("//\"-1.0\" >>> \"NaN\"");
assert_urshift("-1.0", "NaN");

trace("//\"0xFF1306\" >>> \"NaN\"");
assert_urshift("0xFF1306", "NaN");

trace("//true >>> \"-0.0\"");
assert_urshift(true, "-0.0");

trace("//false >>> \"-0.0\"");
assert_urshift(false, "-0.0");

trace("//null >>> \"-0.0\"");
assert_urshift(null, "-0.0");

trace("//undefined >>> \"-0.0\"");
assert_urshift(undefined, "-0.0");

trace("//\"\" >>> \"-0.0\"");
assert_urshift("", "-0.0");

trace("//\"str\" >>> \"-0.0\"");
assert_urshift("str", "-0.0");

trace("//\"true\" >>> \"-0.0\"");
assert_urshift("true", "-0.0");

trace("//\"false\" >>> \"-0.0\"");
assert_urshift("false", "-0.0");

trace("//0.0 >>> \"-0.0\"");
assert_urshift(0.0, "-0.0");

trace("//NaN >>> \"-0.0\"");
assert_urshift(NaN, "-0.0");

trace("//-0.0 >>> \"-0.0\"");
assert_urshift(-0.0, "-0.0");

trace("//Infinity >>> \"-0.0\"");
assert_urshift(Infinity, "-0.0");

trace("//1.0 >>> \"-0.0\"");
assert_urshift(1.0, "-0.0");

trace("//-1.0 >>> \"-0.0\"");
assert_urshift(-1.0, "-0.0");

trace("//0xFF1306 >>> \"-0.0\"");
assert_urshift(0xFF1306, "-0.0");

trace("//new Object() >>> \"-0.0\"");
assert_urshift({}, "-0.0");

trace("//\"0.0\" >>> \"-0.0\"");
assert_urshift("0.0", "-0.0");

trace("//\"NaN\" >>> \"-0.0\"");
assert_urshift("NaN", "-0.0");

trace("//\"-0.0\" >>> \"-0.0\"");
assert_urshift("-0.0", "-0.0");

trace("//\"Infinity\" >>> \"-0.0\"");
assert_urshift("Infinity", "-0.0");

trace("//\"1.0\" >>> \"-0.0\"");
assert_urshift("1.0", "-0.0");

trace("//\"-1.0\" >>> \"-0.0\"");
assert_urshift("-1.0", "-0.0");

trace("//\"0xFF1306\" >>> \"-0.0\"");
assert_urshift("0xFF1306", "-0.0");

trace("//true >>> \"Infinity\"");
assert_urshift(true, "Infinity");

trace("//false >>> \"Infinity\"");
assert_urshift(false, "Infinity");

trace("//null >>> \"Infinity\"");
assert_urshift(null, "Infinity");

trace("//undefined >>> \"Infinity\"");
assert_urshift(undefined, "Infinity");

trace("//\"\" >>> \"Infinity\"");
assert_urshift("", "Infinity");

trace("//\"str\" >>> \"Infinity\"");
assert_urshift("str", "Infinity");

trace("//\"true\" >>> \"Infinity\"");
assert_urshift("true", "Infinity");

trace("//\"false\" >>> \"Infinity\"");
assert_urshift("false", "Infinity");

trace("//0.0 >>> \"Infinity\"");
assert_urshift(0.0, "Infinity");

trace("//NaN >>> \"Infinity\"");
assert_urshift(NaN, "Infinity");

trace("//-0.0 >>> \"Infinity\"");
assert_urshift(-0.0, "Infinity");

trace("//Infinity >>> \"Infinity\"");
assert_urshift(Infinity, "Infinity");

trace("//1.0 >>> \"Infinity\"");
assert_urshift(1.0, "Infinity");

trace("//-1.0 >>> \"Infinity\"");
assert_urshift(-1.0, "Infinity");

trace("//0xFF1306 >>> \"Infinity\"");
assert_urshift(0xFF1306, "Infinity");

trace("//new Object() >>> \"Infinity\"");
assert_urshift({}, "Infinity");

trace("//\"0.0\" >>> \"Infinity\"");
assert_urshift("0.0", "Infinity");

trace("//\"NaN\" >>> \"Infinity\"");
assert_urshift("NaN", "Infinity");

trace("//\"-0.0\" >>> \"Infinity\"");
assert_urshift("-0.0", "Infinity");

trace("//\"Infinity\" >>> \"Infinity\"");
assert_urshift("Infinity", "Infinity");

trace("//\"1.0\" >>> \"Infinity\"");
assert_urshift("1.0", "Infinity");

trace("//\"-1.0\" >>> \"Infinity\"");
assert_urshift("-1.0", "Infinity");

trace("//\"0xFF1306\" >>> \"Infinity\"");
assert_urshift("0xFF1306", "Infinity");

trace("//true >>> \"1.0\"");
assert_urshift(true, "1.0");

trace("//false >>> \"1.0\"");
assert_urshift(false, "1.0");

trace("//null >>> \"1.0\"");
assert_urshift(null, "1.0");

trace("//undefined >>> \"1.0\"");
assert_urshift(undefined, "1.0");

trace("//\"\" >>> \"1.0\"");
assert_urshift("", "1.0");

trace("//\"str\" >>> \"1.0\"");
assert_urshift("str", "1.0");

trace("//\"true\" >>> \"1.0\"");
assert_urshift("true", "1.0");

trace("//\"false\" >>> \"1.0\"");
assert_urshift("false", "1.0");

trace("//0.0 >>> \"1.0\"");
assert_urshift(0.0, "1.0");

trace("//NaN >>> \"1.0\"");
assert_urshift(NaN, "1.0");

trace("//-0.0 >>> \"1.0\"");
assert_urshift(-0.0, "1.0");

trace("//Infinity >>> \"1.0\"");
assert_urshift(Infinity, "1.0");

trace("//1.0 >>> \"1.0\"");
assert_urshift(1.0, "1.0");

trace("//-1.0 >>> \"1.0\"");
assert_urshift(-1.0, "1.0");

trace("//0xFF1306 >>> \"1.0\"");
assert_urshift(0xFF1306, "1.0");

trace("//new Object() >>> \"1.0\"");
assert_urshift({}, "1.0");

trace("//\"0.0\" >>> \"1.0\"");
assert_urshift("0.0", "1.0");

trace("//\"NaN\" >>> \"1.0\"");
assert_urshift("NaN", "1.0");

trace("//\"-0.0\" >>> \"1.0\"");
assert_urshift("-0.0", "1.0");

trace("//\"Infinity\" >>> \"1.0\"");
assert_urshift("Infinity", "1.0");

trace("//\"1.0\" >>> \"1.0\"");
assert_urshift("1.0", "1.0");

trace("//\"-1.0\" >>> \"1.0\"");
assert_urshift("-1.0", "1.0");

trace("//\"0xFF1306\" >>> \"1.0\"");
assert_urshift("0xFF1306", "1.0");

trace("//true >>> \"-1.0\"");
assert_urshift(true, "-1.0");

trace("//false >>> \"-1.0\"");
assert_urshift(false, "-1.0");

trace("//null >>> \"-1.0\"");
assert_urshift(null, "-1.0");

trace("//undefined >>> \"-1.0\"");
assert_urshift(undefined, "-1.0");

trace("//\"\" >>> \"-1.0\"");
assert_urshift("", "-1.0");

trace("//\"str\" >>> \"-1.0\"");
assert_urshift("str", "-1.0");

trace("//\"true\" >>> \"-1.0\"");
assert_urshift("true", "-1.0");

trace("//\"false\" >>> \"-1.0\"");
assert_urshift("false", "-1.0");

trace("//0.0 >>> \"-1.0\"");
assert_urshift(0.0, "-1.0");

trace("//NaN >>> \"-1.0\"");
assert_urshift(NaN, "-1.0");

trace("//-0.0 >>> \"-1.0\"");
assert_urshift(-0.0, "-1.0");

trace("//Infinity >>> \"-1.0\"");
assert_urshift(Infinity, "-1.0");

trace("//1.0 >>> \"-1.0\"");
assert_urshift(1.0, "-1.0");

trace("//-1.0 >>> \"-1.0\"");
assert_urshift(-1.0, "-1.0");

trace("//0xFF1306 >>> \"-1.0\"");
assert_urshift(0xFF1306, "-1.0");

trace("//new Object() >>> \"-1.0\"");
assert_urshift({}, "-1.0");

trace("//\"0.0\" >>> \"-1.0\"");
assert_urshift("0.0", "-1.0");

trace("//\"NaN\" >>> \"-1.0\"");
assert_urshift("NaN", "-1.0");

trace("//\"-0.0\" >>> \"-1.0\"");
assert_urshift("-0.0", "-1.0");

trace("//\"Infinity\" >>> \"-1.0\"");
assert_urshift("Infinity", "-1.0");

trace("//\"1.0\" >>> \"-1.0\"");
assert_urshift("1.0", "-1.0");

trace("//\"-1.0\" >>> \"-1.0\"");
assert_urshift("-1.0", "-1.0");

trace("//\"0xFF1306\" >>> \"-1.0\"");
assert_urshift("0xFF1306", "-1.0");

trace("//true >>> \"0xFF1306\"");
assert_urshift(true, "0xFF1306");

trace("//false >>> \"0xFF1306\"");
assert_urshift(false, "0xFF1306");

trace("//null >>> \"0xFF1306\"");
assert_urshift(null, "0xFF1306");

trace("//undefined >>> \"0xFF1306\"");
assert_urshift(undefined, "0xFF1306");

trace("//\"\" >>> \"0xFF1306\"");
assert_urshift("", "0xFF1306");

trace("//\"str\" >>> \"0xFF1306\"");
assert_urshift("str", "0xFF1306");

trace("//\"true\" >>> \"0xFF1306\"");
assert_urshift("true", "0xFF1306");

trace("//\"false\" >>> \"0xFF1306\"");
assert_urshift("false", "0xFF1306");

trace("//0.0 >>> \"0xFF1306\"");
assert_urshift(0.0, "0xFF1306");

trace("//NaN >>> \"0xFF1306\"");
assert_urshift(NaN, "0xFF1306");

trace("//-0.0 >>> \"0xFF1306\"");
assert_urshift(-0.0, "0xFF1306");

trace("//Infinity >>> \"0xFF1306\"");
assert_urshift(Infinity, "0xFF1306");

trace("//1.0 >>> \"0xFF1306\"");
assert_urshift(1.0, "0xFF1306");

trace("//-1.0 >>> \"0xFF1306\"");
assert_urshift(-1.0, "0xFF1306");

trace("//0xFF1306 >>> \"0xFF1306\"");
assert_urshift(0xFF1306, "0xFF1306");

trace("//new Object() >>> \"0xFF1306\"");
assert_urshift({}, "0xFF1306");

trace("//\"0.0\" >>> \"0xFF1306\"");
assert_urshift("0.0", "0xFF1306");

trace("//\"NaN\" >>> \"0xFF1306\"");
assert_urshift("NaN", "0xFF1306");

trace("//\"-0.0\" >>> \"0xFF1306\"");
assert_urshift("-0.0", "0xFF1306");

trace("//\"Infinity\" >>> \"0xFF1306\"");
assert_urshift("Infinity", "0xFF1306");

trace("//\"1.0\" >>> \"0xFF1306\"");
assert_urshift("1.0", "0xFF1306");

trace("//\"-1.0\" >>> \"0xFF1306\"");
assert_urshift("-1.0", "0xFF1306");

trace("//\"0xFF1306\" >>> \"0xFF1306\"");
assert_urshift("0xFF1306", "0xFF1306");