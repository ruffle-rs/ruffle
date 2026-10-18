var original = flash.external.ExternalInterface;

function describe(value) {
    if (typeof value === "string") {
        return "\"" + value + "\"";
    }
    if (value instanceof Array) {
        return "[" + describeList(value) + "]";
    }
    if (value.nodeName !== undefined) {
        return "<" + value.nodeName + "> node";
    }
    if (typeof value === "function") {
        return "[function]";
    }
    return String(value);
}

function describeList(list) {
    var described = [];
    for (var i = 0; i < list.length; i++) {
        described.push(describe(list[i]));
    }
    return described.join(", ");
}

function addTracingProperty(name, obj, prop, value) {
    obj.addProperty(prop, function() {
        trace("  get " + name + "." + prop);
        return value;
    }, function(newValue) {
        trace("  set " + name + "." + prop + " = " + newValue);
        value = newValue;
    });
}

function addTracingMethod(name, obj, method, result) {
    addTracingProperty(name, obj, method, function() {
        trace("  call " + name + "." + method + "(" + describeList(arguments) + ")");
        if (typeof result === "function") {
            return result.apply(this, arguments);
        }
        return result;
    });
}

function xml(source) {
    var doc = new XML();
    doc.parseXML(source);
    return doc.firstChild;
}

var instance = {};
function method() {
    trace("  method(" + describeList(arguments) + ")");
    return "method result";
}
var callback;

var tracingExternalInterface = {};
addTracingProperty("ExternalInterface", tracingExternalInterface, "available", true);
addTracingMethod("ExternalInterface", tracingExternalInterface, "_initJS", undefined);
addTracingMethod("ExternalInterface", tracingExternalInterface, "_useSetReturnValueHack", true);
addTracingMethod("ExternalInterface", tracingExternalInterface, "_objectID", "objectID");
addTracingMethod("ExternalInterface", tracingExternalInterface, "_toJS", "js");
addTracingMethod("ExternalInterface", tracingExternalInterface, "_evalJS", null);
addTracingMethod("ExternalInterface", tracingExternalInterface, "_argumentsToXML", "<arguments/>");
addTracingMethod("ExternalInterface", tracingExternalInterface, "_callOut", "<number>5</number>");
addTracingMethod("ExternalInterface", tracingExternalInterface, "_toAS", "_toAS result");
addTracingMethod("ExternalInterface", tracingExternalInterface, "_addCallback", function(name, registered) {
    callback = registered;
    return "_addCallback result";
});
addTracingMethod("ExternalInterface", tracingExternalInterface, "_callIn", function(callInstance, callMethod) {
    trace("  instance is the instance: " + (callInstance === instance));
    trace("  method is the method: " + (callMethod === method));
    return "_callIn result";
});
addTracingMethod("ExternalInterface", tracingExternalInterface, "_escapeXML", "_escapeXML result");
addTracingMethod("ExternalInterface", tracingExternalInterface, "_unescapeXML", "_unescapeXML result");
addTracingMethod("ExternalInterface", tracingExternalInterface, "_jsQuoteString", "_jsQuoteString result");
addTracingMethod("ExternalInterface", tracingExternalInterface, "_toXML", "_toXML result");
addTracingMethod("ExternalInterface", tracingExternalInterface, "_arrayToXML", "_arrayToXML result");
addTracingMethod("ExternalInterface", tracingExternalInterface, "_objectToXML", "_objectToXML result");
addTracingMethod("ExternalInterface", tracingExternalInterface, "_arrayToAS", "_arrayToAS result");
addTracingMethod("ExternalInterface", tracingExternalInterface, "_objectToAS", "_objectToAS result");
addTracingMethod("ExternalInterface", tracingExternalInterface, "_argumentsToAS", ["_argumentsToAS result"]);
addTracingMethod("ExternalInterface", tracingExternalInterface, "_arrayToJS", "_arrayToJS result");
addTracingMethod("ExternalInterface", tracingExternalInterface, "_objectToJS", "_objectToJS result");
tracingExternalInterface.__resolve = function(name) {
    trace("  get missing ExternalInterface." + name);
};
flash.external.ExternalInterface = tracingExternalInterface;

trace("// call(\"f\", 1, \"two\")");
trace(original.call("f", 1, "two"));
trace("");

trace("// addCallback(\"name\", instance, method)");
trace(original.addCallback("name", instance, method));
trace("");

trace("// callback(\"<invoke/>\")");
trace(callback("<invoke/>"));
trace("");

trace("// _toXML(\"a<b\")");
trace(original._toXML("a<b"));
trace("");

trace("// _toXML([1])");
trace(original._toXML([1]));
trace("");

trace("// _toXML({})");
trace(original._toXML({}));
trace("");

trace("// _arrayToXML([1])");
trace(original._arrayToXML([1]));
trace("");

trace("// _objectToXML({k: 1})");
trace(original._objectToXML({k: 1}));
trace("");

trace("// _argumentsToXML([\"f\", 1])");
trace(original._argumentsToXML(["f", 1]));
trace("");

trace("// _toAS(<string>a&amp;b</string>)");
trace(original._toAS(xml("<string>a&amp;b</string>")));
trace("");

trace("// _toAS(<array/>)");
trace(original._toAS(xml("<array/>")));
trace("");

trace("// _toAS(<object/>)");
trace(original._toAS(xml("<object/>")));
trace("");

trace("// _arrayToAS(<array><property id=\"0\"><number>1</number></property></array>)");
trace(original._arrayToAS(xml("<array><property id=\"0\"><number>1</number></property></array>")));
trace("");

trace("// _objectToAS(<object><property id=\"k\"><number>1</number></property></object>)");
trace(original._objectToAS(xml("<object><property id=\"k\"><number>1</number></property></object>")).k);
trace("");

trace("// _argumentsToAS(<arguments><number>1</number></arguments>)");
trace(original._argumentsToAS(xml("<arguments><number>1</number></arguments>")));
trace("");

trace("// _toJS(\"a\")");
trace(original._toJS("a"));
trace("");

trace("// _toJS([1])");
trace(original._toJS([1]));
trace("");

trace("// _toJS({})");
trace(original._toJS({}));
trace("");

trace("// _arrayToJS([1])");
trace(original._arrayToJS([1]));
trace("");

trace("// _objectToJS({k: 1})");
trace(original._objectToJS({k: 1}));
trace("");

trace("// _callIn(instance, method, <invoke returntype=\"xml\">)");
trace(original._callIn(instance, method, "<invoke name=\"f\" returntype=\"xml\"><arguments/></invoke>"));
trace("");

trace("// _callIn(instance, method, <invoke returntype=\"javascript\">)");
trace(original._callIn(instance, method, "<invoke name=\"f\" returntype=\"javascript\"><arguments/></invoke>"));

flash.external.ExternalInterface = original;
