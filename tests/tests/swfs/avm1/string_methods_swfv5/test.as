var objToString = {toString: function() { return "toString"; }};

function test() {
	// length
	trace("// length");
	trace('// "foo".length');
	trace("foo".length);

	trace('// "".length')
	trace("".length);

	trace('// "undefined".length');
	trace("undefined".length);

	trace('// "hello".length');
	trace("hello".length);
	trace("");


	// charAt
	trace("// charAt");

	var s = "foo+foo";

	trace("// s.charAt(0)");
	trace(s.charAt(0));

	trace("// s.charAt(1)");
	trace(s.charAt(1));

	trace("// s.charAt(3)");
	trace(s.charAt(3));

	trace("// s.charAt(4)");
	trace(s.charAt(4));

	trace("// s.charAt(5)");
	trace(s.charAt(5));

	trace("// s.charAt(-100)");
	trace(s.charAt(-100));

	trace("// s.charAt(4294967297)");
	trace(s.charAt(4294967297));
	trace("");



	// concat
	trace("// concat");
	var s = "foo";
	trace('// s.concat(s)');
	trace(s.concat(s));

	trace('// s.concat(s, s, s)');
	trace(s.concat(s, s, s));

	trace('// s.concat()');
	trace(s.concat());

	trace('// s.concat(null, s, undefined, 0, objToString, true)');
	trace(s.concat(null, s, undefined, 0, objToString, true));
	trace("");



	// fromCharCode
	trace("// fromCharCode");

	trace('// String.fromCharCode(80)');
	trace(String.fromCharCode(80));

	trace('// String.fromCharCode(-65456)');
	trace(String.fromCharCode(-65456));

	trace('// String.fromCharCode("BAD")');
	trace(String.fromCharCode("BAD"));

	trace('// String.fromCharCode(NaN)');
	trace(String.fromCharCode(NaN));

	trace('// String.fromCharCode()');
	trace(String.fromCharCode());

	trace('// String.fromCharCode(80, 81, 82)');
	trace(String.fromCharCode(80, 81, 82));

	trace('// String.fromCharCode(80, 0, 82)');
	trace(String.fromCharCode(80, 0, 82));
	trace("");

	// charCodeAt
	trace('// charCodeAt');
	var s = "foo+foo";

	trace('// s.charCodeAt(0)');
	trace(s.charCodeAt(0));

	trace('// s.charCodeAt(3)');
	trace(s.charCodeAt(3));

	trace('// s.charCodeAt(4)');
	trace(s.charCodeAt(4));

	trace('// s.charCodeAt(5)');
	trace(s.charCodeAt(5));

	trace('// s.charCodeAt(6)');
	trace(s.charCodeAt(6));

	trace('// s.charCodeAt()');
	trace(s.charCodeAt());

	trace('// s.charCodeAt(-1)');
	trace(s.charCodeAt(-1));

	trace('// s.charCodeAt(9)');
	trace(s.charCodeAt(9));

	trace('// s.charCodeAt("1")');
	trace(s.charCodeAt("1"));

	trace('// s.charCodeAt(undefined)');
	trace(s.charCodeAt(undefined));

	trace('// s.charCodeAt(NaN)');
	trace(s.charCodeAt(NaN));

	trace("");

	// indexOf
	trace("// indexOf ");
	var s = "aaatestFOOtestaaanull";

	trace('// s.indexOf("a")');
	trace(s.indexOf("a"));

	trace('// s.indexOf("a", 16)');
	trace(s.indexOf("a", 16));

	trace('// s.indexOf("a", 14)');
	trace(s.indexOf("a", 14));

	trace('// s.indexOf("a", 13)');
	trace(s.indexOf("a", 13));

	trace('// s.indexOf("a", 0)');
	trace(s.indexOf("a", 0));

	trace('// s.indexOf("test")');
	trace(s.indexOf("test"));

	trace('// s.indexOf("test", 4)');
	trace(s.indexOf("test", 4));

	trace('// s.indexOf("test", 100)');
	trace(s.indexOf("test", 100));

	trace('// s.indexOf("test", -1)');
	trace(s.indexOf("test", -1));

	trace('// s.indexOf("test", 4294967300))');
	trace(s.indexOf("test", 4294967300));

	trace('// s.indexOf("test", null)');
	trace(s.indexOf("test", null));

	trace('// s.indexOf("test", undefined)');
	trace(s.indexOf("test", undefined));

	trace('// s.indexOf("")');
	trace(s.indexOf(""));

	trace('// s.indexOf("", 5)');
	trace(s.indexOf("", 5));

	trace('// s.indexOf("", 100)');
	trace(s.indexOf("", 100));

	trace('// s.indexOf()');
	trace(s.indexOf());

	trace('// s.indexOf(null)');
	trace(s.indexOf(null));
	trace("");

	trace('// s.indexOf(undefined)');
	trace(s.indexOf(undefined));
	trace("");

	trace('// \"hello undefined hi\".indexOf(undefined)');
	trace("hello undefined hi".indexOf(undefined));
	trace("");



	// lastIndexOf
	trace("// lastIndexOf");
	var s = "aaatestFOOtestaaanull";

	trace('// s.lastIndexOf("a")');
	trace(s.lastIndexOf("a"));

	trace('// s.lastIndexOf("a", 16)');
	trace(s.lastIndexOf("a", 16));

	trace('// s.lastIndexOf("a", 14)');
	trace(s.lastIndexOf("a", 14));

	trace('// s.lastIndexOf("a", 13)');
	trace(s.lastIndexOf("a", 13));

	trace('// s.lastIndexOf("a", 0)');
	trace(s.lastIndexOf("a", 0));

	trace('// s.lastIndexOf("test")');
	trace(s.lastIndexOf("test"));

	trace('// s.lastIndexOf("test", 11)');
	trace(s.lastIndexOf("test", 11));

	trace('// s.lastIndexOf("test", 100)');
	trace(s.lastIndexOf("test", 100));

	trace('// s.lastIndexOf("test", -1)');
	trace(s.lastIndexOf("test", -1));

	trace('// s.lastIndexOf("test", null)');
	trace(s.lastIndexOf("test", null));

	trace('// s.lastIndexOf("test", undefined)');
	trace(s.lastIndexOf("test", undefined));

	trace('// s.lastIndexOf("")');
	trace(s.lastIndexOf(""));

	trace('// s.lastIndexOf("", 3)');
	trace(s.lastIndexOf("", 3));

	trace('// s.lastIndexOf("", 21)');
	trace(s.lastIndexOf("", 21));

	trace('// s.lastIndexOf("", 0)');
	trace(s.lastIndexOf("", 0));

	trace('// s.lastIndexOf()');
	trace(s.lastIndexOf());

	trace('// s.lastIndexOf(null)');
	trace(s.lastIndexOf(null));
	trace("");

	trace('// s.lastIndexOf(undefined)');
	trace(s.lastIndexOf(undefined));
	trace("");

	trace('// \"hello undefined hi\".lastIndexOf(undefined)');
	trace("hello undefined hi".lastIndexOf(undefined));
	trace("");


	// slice
	trace("// slice");
	var s = "Hello1234";

	trace('// s.slice(1, 4)');
	trace(s.slice(1, 4));

	trace('// s.slice(3, 1)');
	trace(s.slice(3, 1));

	trace('// s.slice(-3, -1)');
	trace(s.slice(-3, -1));

	trace('// s.slice(-1, -3)');
	trace(s.slice(-1, -3));

	trace('// s.slice()');
	trace(s.slice());

	trace('// s.slice(null, null)');
	trace(s.slice(null, null));

	trace('// s.slice(undefined, undefined)');
	trace(s.slice(undefined, undefined));
	trace("");



	// substr
	trace("// substr");
	var s = "HELLOhello";

	trace('// s.substr(1, 2)');
	trace(s.substr(1, 2));

	trace('// s.substr(1)');
	trace(s.substr(1));

	trace('// s.substr(-5)');
	trace(s.substr(-5));

	trace('// s.substr(6, 3)');
	trace(s.substr(6, 3));

	trace('// s.substr(3, 0)');
	trace(s.substr(3, 0));

	trace('// s.substr(3, undefined)');
	trace(s.substr(3, undefined));

	trace('// s.substr(3, null)');
	trace(s.substr(3, null));

	trace('// s.substr(null, 4)');
	trace(s.substr(null, 4));

	trace('// s.substr(4294967296, -4294967294)');
	trace(s.substr(4294967296, -4294967294));

	trace('// s.substr()');
	trace(s.substr());

	trace('// s.substr(undefined, undefined)');
	trace(s.substr(undefined, undefined));

	trace('// s.substr(null, undefined)');
	trace(s.substr(null, undefined));

	trace('// s.substr(null, null)');
	trace(s.substr(null, null));
	trace("");

	// substring
	trace("// substring");
	var s = "HELLO1hello";

	trace('// s.substring(1, 2)');
	trace(s.substring(1, 2));

	trace('// s.substring(1)');
	trace(s.substring(1));

	trace('// s.substring(-5)');
	trace(s.substring(-5));

	trace('// s.substring(6, 3)');
	trace(s.substring(6, 3));

	trace('// s.substring(3, 0)');
	trace(s.substring(3, 0));

	trace('// s.substring(3, undefined)');
	trace(s.substring(3, undefined));

	trace('// s.substring(3, null)');
	trace(s.substring(3, null));

	trace('// s.substring(null, 4)');
	trace(s.substring(null, 4));

	trace('// s.substring(4294967296, -4294967294)');
	trace(s.substring(4294967296, -4294967294));

	trace('// s.substring(');
	trace(s.substring());

	trace('// s.substring(undefined, undefined)');
	trace(s.substring(undefined, undefined));

	trace('// s.substring(null, undefined)');
	trace(s.substring(null, undefined));

	trace('// s.substring(null, null)');
	trace(s.substring(null, null));
	trace("");



	// split
	trace("// split");
	var s = "A,,b,undefined0,c,";

	trace('// s.split(",")');
	var a = s.split(",");
	trace(a.length);
	trace(a);

	trace('// s.split(",", 2)');
	var a = s.split(",", 2);
	trace(a.length);
	trace(a);

	trace('// s.split(",", 0)');
	var a = s.split(",", 0);
	trace(a.length);
	trace(a);

	trace('// s.split(",", -100)');
	var a = s.split(",", -100);
	trace(a.length);
	trace(a);

	trace('// s.split(",", undefined)');
	var a = s.split(",", undefined);
	trace(a.length);
	trace(a);

	trace('// s.split(",", null)');
	var a = s.split(",", null);
	trace(a.length);
	trace(a);

	trace('// s.split("")');
	var a = s.split("");
	trace(a.length);
	trace(a);

	trace('// s.split(undefined)');
	var a = s.split(undefined);
	trace(a.length);
	trace(a);

	trace('// s.split(undefined, 0)');
	var a = s.split(undefined, 0);
	trace(a.length);
	trace(a);

	trace('// s.split()');
	var a = s.split();
	trace(a.length);
	trace(a);

	trace("");


	// toLowerCase
	trace("// toLowerCase");

	trace('// "teST".toLowerCase()');
	trace("teST".toLowerCase());
	trace('// All uppercase chars');
	trace("ABCDEFGHIJKLMNOPQRSTUVWXYZ".toLowerCase());
	trace("");



	// toUpperCase
	trace("// toUpperCase");
	trace('// "teST".toUpperCase()');
	trace("teST".toUpperCase());
	trace('// All lowercase chars');
	var s = "abcdefghijklmnopqrstuvwxyz";
	trace(s.toUpperCase());
	trace("");
}

test();
