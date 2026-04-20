function test(value) {
    trace(value + " (" + (typeof value) + "):");
    var res = flash.external.ExternalInterface._arrayToXML(value);
    trace("  _arrayToXML: " + value + " -> " + res + " (" + (typeof res) + ")");
    var res = flash.external.ExternalInterface._argumentsToXML(value);
    trace("  _argumentsToXML: " + value + " -> " + res + " (" + (typeof res) + ")");
    var res = flash.external.ExternalInterface._objectToXML(value);
    trace("  _objectToXML: " + value + " -> " + res + " (" + (typeof res) + ")");
    var res = flash.external.ExternalInterface._toXML(value);
    trace("  _toXML: " + value + " -> " + res + " (" + (typeof res) + ")");
}

function testEmpty() {
    var res = flash.external.ExternalInterface._arrayToXML();
    trace("  _arrayToXML: " + res + " (" + (typeof res) + ")");
    var res = flash.external.ExternalInterface._argumentsToXML();
    trace("  _argumentsToXML: " + res + " (" + (typeof res) + ")");
    var res = flash.external.ExternalInterface._objectToXML();
    trace("  _objectToXML: " + res + " (" + (typeof res) + ")");
    var res = flash.external.ExternalInterface._toXML();
    trace("  _toXML: " + res + " (" + (typeof res) + ")");
}

testEmpty();

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
test([]);
test([true]);
test([false]);
test([null]);
test([undefined]);
test([""]);
test([0]);
test([0.0]);
test([5.6]);
test([new Object()]);
test([[]]);
test([[true]]);
test("");
test("test");

var o = new Object();
o.x = "y";
test(o);

o = new Object();
o.x = "y";
ASSetPropFlags(o, "x", 3, 0);
test(o);

o = new Object();
o.x = "&amp;<test>";
test(o);

test("&amp;<test>");

o = new Array();
o[5] = 5;
o[2] = 2;
o[3] = 3;
test(o);

o = new Object();
o["<>&test;\"'"] = "<>&test;\"'";
test(o);

function f1() {
    test(arguments);
}
f1();
f1(undefined);
f1([], true, "");

function f2(val) {
    test(arguments);
}

test(f1);
test(f2);
