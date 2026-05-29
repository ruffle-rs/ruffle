function testAdd(a, b) {
	var aStr = a.name != undefined ? a.name : String(a);
	var bStr = b.name != undefined ? b.name : String(b);
	trace("// " + aStr + " + " + bStr);
	trace( a + b );
	trace("// " + bStr + " + " + aStr);
	trace( b + a );
	trace("");
}

var obj = {name: "obj"};
var objValue1 = {name: "objValue1", valueOf: function() { trace("objValue1.valueOf"); return 1; }, toString: function() { trace("objValue1.toString"); } };
var objValue2 = {name: "objValue2", valueOf: function() { trace("objValue2.valueOf"); return "xyz"; }, toString: function() { trace("objValue2.toString"); } };
var objValue3 = {name: "objValue3", valueOf: function() { trace("objValue3.valueOf"); return {}; }, toString: function() { trace("objValue3.toString"); } };
var objValue4 = {name: "objValue4", valueOf: function() { trace("objValue4.valueOf"); }, toString: function() { trace("objValue4.toString"); } };

testAdd(0, 0);
testAdd(1, 2);
testAdd(NaN, 1);
testAdd(NaN, NaN);
testAdd(Infinity, Infinity);
testAdd(Infinity, -Infinity);

testAdd(false, false);
testAdd(true, false);
testAdd(true, true);
testAdd(true, false);
testAdd(false, 1);
testAdd(true, 1);

testAdd("abc", 1);
testAdd("abc", false);
testAdd("abc", true);
testAdd("500", 1);
testAdd("500", false);
testAdd("500", true);

testAdd(undefined, undefined);
testAdd(undefined, 1);
testAdd(undefined, "abc");

testAdd(null, 1);
testAdd(null, null);
testAdd(null, undefined);
testAdd(null, false);
testAdd(null, true);
testAdd(null, "abc");

testAdd(_root, 1);
testAdd(_root, false);
testAdd(_root, true);
testAdd(_root, "abc");
testAdd(_root, undefined);

testAdd(obj, 1);
testAdd(obj, false);
testAdd(obj, true);
testAdd(obj, "abc");
testAdd(obj, undefined);
testAdd(obj, null);
testAdd(obj, _root);
testAdd(obj, obj);

testAdd(objValue1, 1);
testAdd(objValue1, false);
testAdd(objValue1, true);
testAdd(objValue1, "abc");
testAdd(objValue1, undefined);
testAdd(objValue1, null);
testAdd(objValue1, _root);
testAdd(objValue1, obj);

testAdd(objValue1, obj);
testAdd(objValue1, objValue1);
testAdd(objValue1, objValue2);
testAdd(objValue1, objValue3);
testAdd(objValue1, objValue4);
testAdd(objValue2, obj);
testAdd(objValue2, objValue2);
testAdd(objValue2, objValue3);
testAdd(objValue3, obj);
testAdd(objValue3, objValue4);
testAdd(objValue4, obj);

fscommand("quit");
