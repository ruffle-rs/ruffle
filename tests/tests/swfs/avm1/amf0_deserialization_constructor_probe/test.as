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

// Identifies the operation in progress when a getter or constructor is invoked.
var probePhase = "setup";

// ---------------------------------------------------------------------------
// Replacement constructors. They delegate to the original native constructor,
// then put the native object on a probe prototype so it remains usable while
// making the selected constructor/prototype observable.
//
// Preserve the number of arguments when delegating. In particular,
// new Date() and new Date(undefined) have different semantics.
// ---------------------------------------------------------------------------

function ProbeArray(length) {
    trace("CONSTRUCTOR Array; phase=" + probePhase + "; argc=" + arguments.length);
    for (var i = 0; i < arguments.length; i++) {
        trace("CONSTRUCTOR Array; arg" + i + "=" + arguments[i]);
    }

    var result;
    if (arguments.length == 0) {
        result = new OriginalArray();
    } else {
        result = new OriginalArray(length);
    }

    result.__proto__ = ProbeArrayPrototype;
    result.probeConstructor = "Array";
    result.probeArgument = arguments.length > 0 ? length : undefined;
    return result;
}
var ProbeArrayPrototype = ProbeArray.prototype;
ProbeArrayPrototype.__proto__ = OriginalArray.prototype;
ProbeArrayPrototype.constructor = ProbeArray;
ProbeArrayPrototype.probePrototype = "Array prototype";

function ProbeDate(time) {
    trace("CONSTRUCTOR Date; phase=" + probePhase + "; argc=" + arguments.length);
    for (var i = 0; i < arguments.length; i++) {
        trace("CONSTRUCTOR Date; arg" + i + "=" + arguments[i]);
    }

    var result;
    switch (arguments.length) {
        case 0:
            result = new OriginalDate();
            break;
        case 1:
            result = new OriginalDate(arguments[0]);
            break;
        case 2:
            result = new OriginalDate(arguments[0], arguments[1]);
            break;
        case 3:
            result = new OriginalDate(arguments[0], arguments[1], arguments[2]);
            break;
        case 4:
            result = new OriginalDate(arguments[0], arguments[1], arguments[2], arguments[3]);
            break;
        case 5:
            result = new OriginalDate(arguments[0], arguments[1], arguments[2], arguments[3], arguments[4]);
            break;
        case 6:
            result = new OriginalDate(arguments[0], arguments[1], arguments[2], arguments[3], arguments[4], arguments[5]);
            break;
        default:
            result = new OriginalDate(
                arguments[0],
                arguments[1],
                arguments[2],
                arguments[3],
                arguments[4],
                arguments[5],
                arguments[6]
            );
            break;
    }

    result.__proto__ = ProbeDatePrototype;
    result.probeConstructor = "Date";
    result.probeArgument = arguments.length > 0 ? time : undefined;
    return result;
}
var ProbeDatePrototype = ProbeDate.prototype;
ProbeDatePrototype.__proto__ = OriginalDate.prototype;
ProbeDatePrototype.constructor = ProbeDate;
ProbeDatePrototype.probePrototype = "Date prototype";

function ProbeXML(source) {
    trace("CONSTRUCTOR XML; phase=" + probePhase + "; argc=" + arguments.length);
    for (var i = 0; i < arguments.length; i++) {
        trace("CONSTRUCTOR XML; arg" + i + "=" + arguments[i]);
    }

    var result;
    if (arguments.length == 0) {
        result = new OriginalXML();
    } else {
        result = new OriginalXML(source);
    }

    result.__proto__ = ProbeXMLPrototype;
    result.probeConstructor = "XML";
    result.probeArgument = arguments.length > 0 ? source : undefined;
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
            trace("GLOBAL GETTER " + name + "; phase=" + probePhase);
            return replacement;
        },
        null
    );

    var prototypeGetterInstalled = OriginalObject.prototype.addProperty.call(
        replacement,
        "prototype",
        function() {
            trace("PROTOTYPE GETTER " + name + "; phase=" + probePhase);
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
    trace(channel + "." + name + ".isNull=" + (value === null));
    trace(channel + "." + name + ".isUndefined=" + (value === undefined));

    if (value === undefined || value === null) {
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
    if (value === undefined || value === null) {
        return;
    }

    trace(channel + "." + name + ".length=" + value.length);
    trace(channel + "." + name + "[0]=" + value[0]);
    trace(channel + "." + name + "[1]=" + value[1]);
    trace(channel + "." + name + ".extra=" + value.extra);
}

function inspectDate(channel, value) {
    inspectObject(channel, "Date", value, OriginalDate, ProbeDate);
    if (value === undefined || value === null) {
        return;
    }

    trace(channel + ".Date.getTimeType=" + typeof(value.getTime));

    // Call the original native method directly instead of relying on the
    // object's prototype chain. This tests whether the value has native
    // Date internal state even if the deserializer did not install a prototype.
    var dateTime;
    var isNativeDate = true;
    try {
        dateTime = OriginalDate.prototype.getTime.call(value);
    } catch (e) {
        isNativeDate = false;
        trace(channel + ".Date.nativeGetTimeError=" + e);
    }

    trace(channel + ".Date.isNativeDate=" + isNativeDate);
    if (isNativeDate) {
        trace(channel + ".Date.getTime=" + dateTime);
    }
}

function inspectXML(channel, value) {
    inspectObject(channel, "XML", value, OriginalXML, ProbeXML);
    if (value === undefined || value === null) {
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
    probePhase = "LocalConnection callback";
    trace("CALLBACK LocalConnection");
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
        trace("ARRAY PROTOTYPE LENGTH GETTER; phase=" + probePhase);
        throw "Array.prototype.length getter invoked";
    },
    function(value) {
        trace("ARRAY PROTOTYPE LENGTH SETTER; phase=" + probePhase);
        throw "Array.prototype.length setter invoked";
    }
);
trace("GETTER/SETTER INSTALLED Array.prototype.length: " + arrayPrototypeLengthGetterInstalled);

// ---------------------------------------------------------------------------
// Channel 1: LocalConnection. This exercises the LSO-style AMF0 decoder used
// for nested LocalConnection values and SharedObject values.
// ---------------------------------------------------------------------------
trace("BEFORE LocalConnection send");
probePhase = "LocalConnection send";
sender.send("amf0_constructor_probe", "onReceiveData", payload);
trace("AFTER LocalConnection send");
probePhase = "after LocalConnection send";

// ---------------------------------------------------------------------------
// Channel 2: NetConnection. server.py returns an AMF0 response containing the
// same four value types, exercising deserialize_value rather than the LSO path.
// ---------------------------------------------------------------------------
var netConnection = new NetConnection();
var responder = new Object();
responder.onResult = function(received) {
    probePhase = "NetConnection onResult";
    trace("CALLBACK NetConnection onResult");
    inspectPayload("NetConnection", received);
};
responder.onStatus = function(info) {
    probePhase = "NetConnection onStatus";
    trace("NetConnection status=" + info.code);
};

trace("BEFORE NetConnection connect");
probePhase = "NetConnection connect";
netConnection.connect("http://localhost:8000/");
trace("AFTER NetConnection connect");

trace("BEFORE NetConnection call");
probePhase = "NetConnection call";
netConnection.call("probe.amf0ConstructorInstantiation", responder);
trace("AFTER NetConnection call");
probePhase = "after NetConnection call";

