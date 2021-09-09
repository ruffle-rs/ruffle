package {
	public class test {}
}

var ruffle_object = {s: "Ruffle Test Object"};
ruffle_object.toString = function() {
    return this.s;
}
trace("// var s = new String(\"5\");");
var s = new String("5");

trace("// trace(s.concat());");
trace(s.concat());
trace("// trace(s.concat(1));");
trace(s.concat(1));
trace("// trace(s.concat(s));");
trace(s.concat(s));
trace("// trace(s.concat(s, 1));");
trace(s.concat(s, 1));
trace("// trace(s.concat(\"asdf\"));");
trace(s.concat("asdf"));
trace("// trace(s.concat(null, s, undefined, 0, {}, ruffle_object, true));");
trace(s.concat(null, s, undefined, 0, {}, ruffle_object, true));

trace("/// fromCharCode");
trace("// trace(String.fromCharCode);");
trace(String.fromCharCode);
trace("// trace(String.fromCharCode(80));");
trace(String.fromCharCode(80));
trace("// trace(String.fromCharCode(12345));");
trace(String.fromCharCode(12345));
trace("// trace(String.fromCharCode(65616));");
trace(String.fromCharCode(65616));
trace("// trace(String.fromCharCode(-65456));");
trace(String.fromCharCode(-65456));
trace("// trace(String.fromCharCode(0xd801));");
trace(String.fromCharCode(0xd801));
trace("// trace(String.fromCharCode(\"BAD\"));");
trace(String.fromCharCode("BAD"));
trace("// String.fromCharCode(NaN)");
trace(String.fromCharCode(NaN));
trace("// String.fromCharCode()");
trace(String.fromCharCode());
trace("// String.fromCharCode(80, 81, 82)");
trace(String.fromCharCode(80, 81, 82));
trace("// String.fromCharCode(80, 0, 82)");
trace(String.fromCharCode(80, 0, 82));
