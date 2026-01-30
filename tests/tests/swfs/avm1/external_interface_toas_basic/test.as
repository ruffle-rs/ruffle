function traceProps(indent, obj) {
    if (obj.length !== undefined) {
        trace(indent + "(length: " + obj.length + ")");
    }

    for (var p in obj) {
        var line = indent + p;
        if (obj.hasOwnProperty(key)) {
            line += ", own";
        }
        trace(line);
        traceProps(indent + "  ", obj[p]);
    }
}

function test(value, name) {
    var valueStr;
    if (value.nodeName) {
        valueStr = "<" + value.nodeName + "/>";
    } else {
        valueStr = "" + value;
    }

    if (name) {
        valueStr += " (" + name + ")";
    }

    trace(valueStr + " (" + (typeof value) + "):");
    var res = flash.external.ExternalInterface._arrayToAS(value);
    trace("  _arrayToAS: " + value + " -> " + res + " (" + (typeof res) + ")");
    traceProps("    ", res);
    var res = flash.external.ExternalInterface._argumentsToAS(value);
    trace("  _argumentsToAS: " + value + " -> " + res + " (" + (typeof res) + ")");
    traceProps("    ", res);
    var res = flash.external.ExternalInterface._objectToAS(value);
    trace("  _objectToAS: " + value + " -> " + res + " (" + (typeof res) + ")");
    traceProps("    ", res);
    var res = flash.external.ExternalInterface._toAS(value);
    trace("  _toAS: " + value + " -> " + res + " (" + (typeof res) + ")");
    traceProps("    ", res);
}

function testEmpty() {
    var res = flash.external.ExternalInterface._arrayToAS();
    trace("  _arrayToAS: " + res + " (" + (typeof res) + ")");
    traceProps("    ", res);
    var res = flash.external.ExternalInterface._argumentsToAS();
    trace("  _argumentsToAS: " + res + " (" + (typeof res) + ")");
    traceProps("    ", res);
    var res = flash.external.ExternalInterface._objectToAS();
    trace("  _objectToAS: " + res + " (" + (typeof res) + ")");
    traceProps("    ", res);
    var res = flash.external.ExternalInterface._toAS();
    trace("  _toAS: " + res + " (" + (typeof res) + ")");
    traceProps("    ", res);
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

var o = new Object();
o.nodeName = "null";
test(o);

var o = new Object();
o.nodeName = "Null";
test(o);

var o = new Object();
o.nodeName = new String("null");
test(o);

var o = new Object();
var q = new Object();
q.toString = function() { return "null"; };
o.nodeName = q;
test(o);

var o = new Object();
o.nodeName = "true";
test(o);

var o = new Object();
o.nodeName = "TRUE";
test(o);

var o = new Object();
o.nodeName = "false";
test(o);

var o = new Object();
o.nodeName = "undefined";
test(o);

var o = new Object();
o.nodeName = "string";
test(o);

var o = new Object();
o.nodeName = "string";
o.firstChild = "value";
test(o);

var o = new Object();
o.nodeName = "string";
o.firstChild = new String("value2");
test(o);

var o = new Object();
o.nodeName = "string";
var q = new Object();
q.toString = function() { return "value3"; };
o.firstChild = q;
test(o);

var o = new Object();
o.nodeName = "string";
o.firstChild = new Object();
test(o);

var o = new Object();
o.nodeName = "number";
test(o);

var o = new Object();
o.nodeName = "number";
o.firstChild = new Object();
test(o);

var o = new Object();
o.nodeName = "number";
var q = new Object();
q.toString = function() { return "5"; };
o.firstChild = q;
test(o);

var o = new Object();
o.nodeName = "number";
var q = new Object();
q.toString = function() { return "Infinity"; };
o.firstChild = q;
test(o);

var o = new Object();
o.nodeName = "number";
o.firstChild = "Infinity";
test(o);

var o = new Object();
o.nodeName = "number";
o.firstChild = "-6";
test(o);

var o = new Object();
o.nodeName = "number";
o.firstChild = "-2.1";
test(o);

var o = new Object();
o.nodeName = "number";
o.firstChild = "-2.1x";
test(o);

var o = new Object();
o.nodeName = "number";
o.firstChild = " -2.1";
test(o);

var o = new Object();
o.nodeName = "number";
o.firstChild = "-2.1 ";
test(o);

var o = new Object();
o.nodeName = "number";
o.firstChild = "-2 .1";
test(o);
