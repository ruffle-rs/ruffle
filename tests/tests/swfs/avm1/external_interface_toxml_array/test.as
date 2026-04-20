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

test([]);
test("");
test(new String());
test(new String("hello"));

var o = new Object();
o.length = 5;
test(o);
