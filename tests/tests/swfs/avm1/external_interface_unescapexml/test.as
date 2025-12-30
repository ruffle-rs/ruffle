function test(value) {
    var res = flash.external.ExternalInterface._unescapeXML(value);
    trace(value + " -> " + res + " (" + (typeof res) + ")");
}

var res = flash.external.ExternalInterface._unescapeXML();
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
test("a&amp;b");
test("a&quot;b");
test("a&apos;b");
test("a&lt;b");
test("a&gt;b");
test("a&Lt;b");
test("a&lT;b");
test("a&LT;b");
test("a&Gt;b");
test("a&gT;b");
test("a&GT;b");
test("a&Apos;b");
test("a&Quot;b");
test("a&Amp;b");
test("a&Amp;b");
test("a&QUOT;b");
test("a&#x0020;b");

test("&amp;gt;");
test("&amp;amp;");
