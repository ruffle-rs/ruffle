// Probe how Flash instantiates AMF0 StrictArray, ECMAArray, Date, and XML values.
//
// Each replacement class has:
//   * a constructor trace (did Flash invoke the constructor?);
//   * a distinctive prototype marker (which prototype was installed?);
//   * a getter on its global binding (did Flash resolve the class through a getter?);
//   * a getter on its `prototype` property (did Flash retrieve the prototype normally?).
//
// The input values are created before monkey-patching so the sender still serializes
// genuine native arrays, dates, and XML objects.

var OriginalObject = Object;
var OriginalArray = Array;
var OriginalDate = Date;
var OriginalXML = XML;

// ---------------------------------------------------------------------------
// Replacement constructors. They delegate to the original native constructor,
// then put the native object on a probe prototype so it remains usable while
// making the selected constructor/prototype observable.
// ---------------------------------------------------------------------------

function ProbeArray(length) {
    trace("CONSTRUCTOR Array; argc=" + arguments.length + "; arg0=" + length);
    var result = new OriginalArray(length);
    result.__proto__ = ProbeArrayPrototype;
    result.probeConstructor = "Array";
    result.probeArgument = length;
    return result;
}
var ProbeArrayPrototype = ProbeArray.prototype;
ProbeArrayPrototype.__proto__ = OriginalArray.prototype;
ProbeArrayPrototype.constructor = ProbeArray;
ProbeArrayPrototype.probePrototype = "Array prototype";

function ProbeDate(time) {
    trace("CONSTRUCTOR Date; argc=" + arguments.length + "; arg0=" + time);
    var result = new OriginalDate(time);
    result.__proto__ = ProbeDatePrototype;
    result.probeConstructor = "Date";
    result.probeArgument = time;
    return result;
}
var ProbeDatePrototype = ProbeDate.prototype;
ProbeDatePrototype.__proto__ = OriginalDate.prototype;
ProbeDatePrototype.constructor = ProbeDate;
ProbeDatePrototype.probePrototype = "Date prototype";

function ProbeXML(source) {
    trace("CONSTRUCTOR XML; argc=" + arguments.length + "; arg0=" + source);
    var result = new OriginalXML(source);
    result.__proto__ = ProbeXMLPrototype;
    result.probeConstructor = "XML";
    result.probeArgument = source;
    return result;
}
var ProbeXMLPrototype = ProbeXML.prototype;
ProbeXMLPrototype.__proto__ = OriginalXML.prototype;
ProbeXMLPrototype.constructor = ProbeXML;
ProbeXMLPrototype.probePrototype = "XML prototype";

// Install both an ordinary stored value and a virtual getter, matching the
// monkey-patching technique in tests/tests/swfs/avm1/globals_monkeypatch.
function instrumentClass(name, replacement, replacementPrototype) {
    _global[name] = replacement;

    var globalGetterInstalled = OriginalObject.prototype.addProperty.call(
        _global,
        name,
        function() {
            trace("GLOBAL GETTER " + name);
            return replacement;
        },
        null
    );

    var prototypeGetterInstalled = OriginalObject.prototype.addProperty.call(
        replacement,
        "prototype",
        function() {
            trace("PROTOTYPE GETTER " + name);
            return replacementPrototype;
        },
        null
    );

    trace("GETTERS INSTALLED " + name + ": global=" + globalGetterInstalled +
        ", prototype=" + prototypeGetterInstalled);
}

// ---------------------------------------------------------------------------
// Inspectors. Do not use `instanceof ProbeX`: that may itself read ProbeX's
// instrumented `prototype` property and add misleading getter traces.
// ---------------------------------------------------------------------------

function inspectObject(channel, name, value, originalConstructor, probeConstructor) {
    trace(channel + "." + name + ".type=" + typeof(value));
    if (value == undefined || value == null) {
        return;
    }

    trace(channel + "." + name + ".probeConstructor=" + value.probeConstructor);
    trace(channel + "." + name + ".probePrototype=" + value.probePrototype);
    trace(channel + "." + name + ".constructorIsProbe=" +
        (value.constructor === probeConstructor));
    trace(channel + "." + name + ".instanceofOriginal=" +
        (value instanceof originalConstructor));
    trace(channel + "." + name + ".probeArgument=" + value.probeArgument);
}

function inspectArray(channel, name, value, originalConstructor, probeConstructor) {
    inspectObject(channel, name, value, originalConstructor, probeConstructor);
    if (value == undefined || value == null) {
        return;
    }

    trace(channel + "." + name + ".length=" + value.length);
    trace(channel + "." + name + "[0]=" + value[0]);
    trace(channel + "." + name + "[1]=" + value[1]);
    trace(channel + "." + name + ".extra=" + value.extra);
}

function inspectDate(channel, value) {
    inspectObject(channel, "Date", value, OriginalDate, ProbeDate);
    if (value == undefined || value == null) {
        return;
    }

    trace(channel + ".Date.getTimeType=" + typeof(value.getTime));
    if (typeof(value.getTime) == "function") {
        trace(channel + ".Date.getTime=" + value.getTime());
    }
}

function inspectXML(channel, value) {
    inspectObject(channel, "XML", value, OriginalXML, ProbeXML);
    if (value == undefined || value == null) {
        return;
    }

    trace(channel + ".XML.firstChildType=" + typeof(value.firstChild));
    if (value.firstChild != undefined && value.firstChild != null) {
        trace(channel + ".XML.root=" + value.firstChild.nodeName);
    }
    trace(channel + ".XML.toStringType=" + typeof(value.toString));
    if (typeof(value.toString) == "function") {
        trace(channel + ".XML.toString=" + value.toString());
    }
}

function inspectPayload(channel, value) {
    trace("========== " + channel + " ==========");
    inspectArray(channel, "StrictArray", value.strictArray, OriginalArray, ProbeArray);
    inspectArray(channel, "ECMAArray", value.ecmaArray, OriginalArray, ProbeArray);
    inspectDate(channel, value.date);
    inspectXML(channel, value.xml);
}

// ---------------------------------------------------------------------------
// Create the source values before replacing the global constructors.
// The associative property forces LocalConnection to encode ecmaArray as an
// ECMAArray, while strictArray remains a StrictArray.
// ---------------------------------------------------------------------------

var strictArray = new OriginalArray(2);
strictArray[0] = "strict-zero";
strictArray[1] = "strict-one";

var ecmaArray = new OriginalArray(1);
ecmaArray[0] = "ecma-zero";
ecmaArray.extra = "ecma-extra";

var payload = {
    strictArray: strictArray,
    ecmaArray: ecmaArray,
    date: new OriginalDate(1234567890000),
    xml: new OriginalXML("<probe id='xml-probe'><child>xml-value</child></probe>")
};

// The root is deliberately an ordinary object. LocalConnection handles a
// top-level array through a separate path; nested values here go through
// deserialize_lso_value, including both array branches in the TODO block.
var receiver = new LocalConnection();
receiver.onReceiveData = function(received) {
    inspectPayload("LocalConnection", received);
};
trace("LocalConnection connected=" + receiver.connect("amf0_constructor_probe"));
var sender = new LocalConnection();

// Instrument all three global classes after constructing the source values.
instrumentClass("Array", ProbeArray, ProbeArrayPrototype);
instrumentClass("Date", ProbeDate, ProbeDatePrototype);
instrumentClass("XML", ProbeXML, ProbeXMLPrototype);

// AMF array creation must install its own length without invoking inherited
// accessors on the current Array.prototype.
var arrayPrototypeLengthGetterInstalled = OriginalObject.prototype.addProperty.call(
    ProbeArrayPrototype,
    "length",
    function() {
        trace("ARRAY PROTOTYPE LENGTH GETTER");
        throw "Array.prototype.length getter invoked";
    },
    function(value) {
        trace("ARRAY PROTOTYPE LENGTH SETTER");
        throw "Array.prototype.length setter invoked";
    }
);
trace("GETTER/SETTER INSTALLED Array.prototype.length: " + arrayPrototypeLengthGetterInstalled);

// ---------------------------------------------------------------------------
// Channel 1: LocalConnection. This exercises the LSO-style AMF0 decoder used
// for nested LocalConnection values and SharedObject values.
// ---------------------------------------------------------------------------
sender.send("amf0_constructor_probe", "onReceiveData", payload);

// ---------------------------------------------------------------------------
// Channel 2: NetConnection. server.py returns an AMF0 response containing the
// same four value types, exercising deserialize_value rather than the LSO path.
// ---------------------------------------------------------------------------
var netConnection = new NetConnection();
var responder = new Object();
responder.onResult = function(received) {
    inspectPayload("NetConnection", received);
};
responder.onStatus = function(info) {
    trace("NetConnection status=" + info.code);
};
netConnection.connect("http://localhost:8000/");
netConnection.call("probe.amf0ConstructorInstantiation", responder);
