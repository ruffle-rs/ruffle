// Exercise native and script-defined AMF0 Date handling and invalid Date bindings.
trace("--- Starting AMF0 Date deserialization edge-case test ---");

var nativeDateConnection = new NetConnection();
var scriptDateConnection = new NetConnection();
var invalidDateConnection = new NetConnection();
var numberDateConnection = new NetConnection();
var missingDateConnection = new NetConnection();

// A script-defined replacement for the global Date constructor. AMF0 Date
// deserialization should use this only for its stored prototype; it must not
// invoke this constructor or install native Date state.
var scriptDateConstructorCalls = 0;
var scriptDateConstructorArguments = -1;

function ScriptDate() {
    scriptDateConstructorCalls++;
    scriptDateConstructorArguments = arguments.length;
    this.scriptDateMarker = "script-date-object";
}

ScriptDate.prototype.scriptDatePrototypeMarker = "script-date-prototype";

var nonCallableDateResponder = new Object();
nonCallableDateResponder.onResult = function(received) {
    if (typeof(received.date) != "undefined") {
        trace("Unexpected value for non-callable Date: " + received.date);
    }

    // Number is a native constructor, but it does not create NativeObject::Date.
    // The AMF0 timestamp must not be written into the Number object.
    _global.Date = Number;
    numberDateConnection.connect("http://localhost:8000/test2");
    numberDateConnection.call("test.nativeDate", numberDateResponder);
};

var numberDateResponder = new Object();
numberDateResponder.onResult = function(received) {
    var date = received.date;
    if (!(date instanceof Number) || date.valueOf() != 0) {
        trace("Unexpected value for native non-Date constructor: " + date);
    }
    // An undefined global binding cannot be resolved as a Date constructor.
    _global.Date = undefined;
    missingDateConnection.connect("http://localhost:8000/test2");
    missingDateConnection.call("test.nativeDate", missingDateResponder);
};

var scriptDateResponder = new Object();
scriptDateResponder.onResult = function(received) {
    var date = received.date;

    trace("--- Script-defined AMF0 Date ---");
    trace("Date type: " + typeof(date));
    trace("Constructor calls: " + scriptDateConstructorCalls);
    trace("Constructor argument count: " + scriptDateConstructorArguments);
    trace("Date marker: " + date.scriptDateMarker);
    trace("Date prototype marker: " + date.scriptDatePrototypeMarker);
    trace("Date getTime type: " + typeof(date.getTime));
    trace("Date is ScriptDate: " + (date instanceof ScriptDate));

    // Check both a non-callable Date binding and a different native
    // constructor. These exercise the fallback and non-Date native-object
    // paths without changing the Flash-observed script-defined Date case.
    _global.Date = new Object();
    invalidDateConnection.connect("http://localhost:8000/test2");
    invalidDateConnection.call("test.nativeDate", nonCallableDateResponder);
};

var nativeDateResponder = new Object();
nativeDateResponder.onResult = function(received) {
    trace("--- Native AMF0 Date ---");
    trace("Date type: " + typeof(received.date));
    trace("Date is native: " + (received.date instanceof Date));
    trace("Date getTime: " + received.date.getTime());

    // Reuse the same AMF0 Date response with a script-defined global Date.
    // This exercises the non-NativeObject::Date branch in date::from_amf0.
    _global.Date = ScriptDate;
    scriptDateConnection.connect("http://localhost:8000/test2");
    scriptDateConnection.call("test.nativeDate", scriptDateResponder);
};

// test2 contains a native AMF0 Date. Do not replace Date before this response:
// the deserializer must construct an actual native Date and restore its time.
nativeDateConnection.connect("http://localhost:8000/test2");
nativeDateConnection.call("test.nativeDate", nativeDateResponder);

var missingDateResponder = new Object();
missingDateResponder.onResult = function(received) {
    trace("--- Missing AMF0 Date binding ---");
    trace("Date type: " + typeof(received.date));
    trace("Date value: " + received.date);
};
