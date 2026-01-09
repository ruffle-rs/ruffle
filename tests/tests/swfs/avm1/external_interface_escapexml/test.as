function test(value) {
    var res = flash.external.ExternalInterface._escapeXML(value);
    trace(value + " -> " + res + " (" + (typeof res) + ")");
}

var res = flash.external.ExternalInterface._escapeXML();
trace(res + " (" + (typeof res) + ")");

test(true);
test(false);
test(null);
test(undefined);
test(new Object());
test(4);
test(-2);
test(1.0/0.0);
test(-1.0/0.0);
test(0.0/0.0);
test("");
test("test");
test("²«¢·ŋœ’ąś„");
test("`~!@#$%^&*()_+-=[]{}\\|;':\",./<>?");
test("<");
test(">");
test("&");
test(";");
test("'");
test("\"");
test("&amp;");
test("&quot;");
test("&apos;");
test("&lt;");
test("&gt;");
