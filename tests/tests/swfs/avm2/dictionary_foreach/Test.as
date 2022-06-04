package {
	public class Test {
	}
}

import flash.utils.Dictionary;

function printKeys(dict:Dictionary):void {
	var keys:Array = new Array();
	for (var k in dict) {
		keys.push(k);
	}
	keys.sort();
	trace("Keys: " + keys);
}

trace("///var a = new Dictionary()");
var a = new Dictionary();

a[new String("foo")] = "The value";
trace("Different string: " + a[new String("foo")]);

var firstKey = new Object();
a[firstKey] = "Testing";
a[1234567] = true;
a.setPropertyIsEnumerable(1234567, false);
trace("Is existent property enumerable: " + a.propertyIsEnumerable(1234567));
trace("Is nonexistent property enumerable " + a.propertyIsEnumerable(new Object()));

a.setPropertyIsEnumerable(firstKey, false);
trace("Showing first key");
printKeys(a);

trace("///a[\"key\"] = 5");
a["key"] = 5;

trace("///a[\"key\"]");
trace(a["key"]);
trace("///a[\"key\"] = 6");
a["key"] = 6;
trace("///var key2 = new Test()");
var key2 = new Test();

trace("///a[key2] = 23");
a[key2] = 23;
trace("///var key3 = new Test()");
var key3 = new Test();

trace('///a[key3] = "Key3 True Value"');
a[key3] = "Key3 True Value";

trace('///a["key3"] = "Key3 False Value"');
a["key3"] = "Key3 False Value";

trace("///var key4 = {\"toString\": function() { return \"key4\"; }}");
var key4 = {"toString": function() { return "key4"; }};

trace('///a[key4] = "Key4 True Value"');
a[key4] = "Key4 True Value";

trace('///a["key4"] = "Key4 False Value"');
a["key4"] = "Key4 False Value";

trace('///a[13] = "i\'ve been found!"');
a[13] = "i've been found!";

trace('///a["13"] = "no I haven\'t"');
a["13"] = "no I haven't";

trace('///a[1.123] = "this violates Rust!"');
a[1.123] = "this violates Rust!";

trace('///a["1.123"] = "this is perfectly acceptable"');
a["1.123"] = "this is perfectly acceptable";

trace('///a[undefined] = "oh no"');
a[undefined] = "oh no";

trace('///a["undefined"] = "uh huh..."');
a["undefined"] = "uh huh...";

trace('///a[null] = "oh YES!"');
a[null] = "oh YES!";

trace('///a["null"] = "yeah sure"');
a["null"] = "yeah sure";

trace('///a[true] = "true"');a[true] = "true";

trace('///a["true"] = "stringy true"');
a["true"] = "stringy true";

trace('///a[false] = "false"');
a[false] = "false";

trace('///a["false"] = "stringy false"');
a["false"] = "stringy false";

trace('///a[a] = a');
a[a] = a;

trace("/// (enumerating object keys...)");
printKeys(a);

trace("///a.setPropertyIsEnumerable(key2, false);");
a.setPropertyIsEnumerable(key2, false);

trace("///a.setPropertyIsEnumerable(key3, false);");
a.setPropertyIsEnumerable(key3, false);

trace("///a.setPropertyIsEnumerable(key4, false);");
a.setPropertyIsEnumerable(key4, false);

trace("/// (enumerating object keys...)");
printKeys(a);