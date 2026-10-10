function keys(obj) {
    var result = "";
    for (var key in obj) {
        result += key + " ";
    }
    return result;
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

function tracingMethod(name, obj, method) {
    var original = obj[method];
    obj[method] = function() {
        trace("  call " + name + "." + method + "(" + arguments.join(", ") + ")");
        return original.apply(this, arguments);
    };
}

function tracingConstructor(name) {
    return function() {
        trace("  new " + name + "(" + arguments.join(", ") + ")");
    };
}

function tracingNumber(name, value) {
    return {valueOf: function() {
        trace("  valueOf " + name);
        return value;
    }};
}

function makeListener(name) {
    var listener = {toString: function() {
        return name;
    }};
    var handler = function() {
        trace("  this === " + name + ": " + (this === listener));
    };
    listener.event1 = handler;
    listener.event2 = handler;
    tracingMethod(name, listener, "event1");
    tracingMethod(name, listener, "event2");
    return listener;
}

function makeBroadcaster() {
    var broadcaster = {};
    AsBroadcaster.initialize(broadcaster);
    return broadcaster;
}

function literal(value) {
    if (typeof value === "string") {
        return "'" + value + "'";
    }
    if (typeof value === "object" && value !== null) {
        return "{}";
    }
    return value;
}

function dumpListeners(broadcaster) {
    var listeners = broadcaster._listeners;
    trace("  _listeners: " + listeners.length + " (" + listeners + ")");
}

var listener1 = makeListener("listener1");
var listener2 = makeListener("listener2");

var methods = ["broadcastMessage", "addListener", "removeListener"];
var staticMethods = methods.concat(["initialize"]);
var members = methods.concat(["_listeners"]);

trace("/// AsBroadcaster");

trace("// typeof AsBroadcaster");
trace(typeof AsBroadcaster);
trace("");

trace("// new AsBroadcaster() instanceof AsBroadcaster");
trace((new AsBroadcaster()) instanceof AsBroadcaster);
trace("");

trace("// AsBroadcaster");
trace(keys(AsBroadcaster));
trace("");

trace("// AsBroadcaster, without DONT_ENUM");
ASSetPropFlags(AsBroadcaster, staticMethods, 0, 1);
trace(keys(AsBroadcaster));
ASSetPropFlags(AsBroadcaster, staticMethods, 1);
trace("");

for (var i = 0; i < staticMethods.length; i++) {
    var method = staticMethods[i];
    trace("// delete AsBroadcaster." + method);
    trace(delete AsBroadcaster[method]);
    trace("");
}
trace("");

trace("/// initialize");

trace("// AsBroadcaster.initialize(broadcaster)");
var broadcaster = {};
trace(AsBroadcaster.initialize(broadcaster));
dumpListeners(broadcaster);
trace("");

trace("// broadcaster");
trace(keys(broadcaster));
trace("");

trace("// broadcaster, without DONT_ENUM");
ASSetPropFlags(broadcaster, members, 0, 1);
trace(keys(broadcaster));
ASSetPropFlags(broadcaster, members, 1);
trace("");

trace("// broadcaster instanceof AsBroadcaster");
trace(broadcaster instanceof AsBroadcaster);
trace("");

trace("// broadcaster._listeners instanceof Array");
trace(broadcaster._listeners instanceof Array);
trace("");

trace("// broadcaster._listeners === makeBroadcaster()._listeners");
trace(broadcaster._listeners === makeBroadcaster()._listeners);
trace("");

for (i = 0; i < methods.length; i++) {
    method = methods[i];
    trace("// broadcaster." + method + " === AsBroadcaster." + method);
    trace(broadcaster[method] === AsBroadcaster[method]);
    trace("");
}

for (i = 0; i < members.length; i++) {
    var member = members[i];
    trace("// delete broadcaster." + member);
    trace(delete broadcaster[member]);
    trace("");
}

var builtins = ["MovieClipLoader.prototype", "Stage", "System.IME", "Key", "Mouse", "Selection", "TextField.prototype", "flash.net.FileReference.prototype", "flash.net.FileReferenceList.prototype"];
for (i = 0; i < builtins.length; i++) {
    var builtin = builtins[i];
    trace("// " + builtin + ".addListener === AsBroadcaster.addListener");
    trace(eval(builtin).addListener === AsBroadcaster.addListener);
    trace("");
}

trace("// AsBroadcaster.initialize(broadcaster), with tracing properties");
broadcaster = {};
for (i = 0; i < members.length; i++) {
    member = members[i];
    addTracingProperty("broadcaster", broadcaster, member);
}
broadcaster.__resolve = function(prop) {
    trace("  resolve broadcaster." + prop);
};
broadcaster.__proto__ = undefined;
AsBroadcaster.initialize(broadcaster);
trace("");

trace("// AsBroadcaster.initialize(broadcaster), with AsBroadcaster replaced");
var originalAsBroadcaster = AsBroadcaster;
var replacedAddListener = function() {
};
var replacedAsBroadcaster = {};
addTracingProperty("AsBroadcaster", replacedAsBroadcaster, "addListener", replacedAddListener);
replacedAsBroadcaster.__resolve = function(prop) {
    trace("  resolve AsBroadcaster." + prop);
};
replacedAsBroadcaster.__proto__ = undefined;
_global.AsBroadcaster = replacedAsBroadcaster;
broadcaster = {};
originalAsBroadcaster.initialize(broadcaster);
_global.AsBroadcaster = originalAsBroadcaster;
trace(broadcaster.addListener === replacedAddListener);
trace("");

trace("// AsBroadcaster.initialize(broadcaster), with Array replaced");
var originalArray = Array;
var tracingArray = tracingConstructor("Array");
_global.Array = tracingArray;
broadcaster = makeBroadcaster();
_global.Array = originalArray;
trace(broadcaster._listeners.__proto__ === tracingArray.prototype);
trace("");
trace("");

trace("/// addListener");

broadcaster = makeBroadcaster();

trace("// broadcaster.addListener(listener1)");
trace(broadcaster.addListener(listener1));
dumpListeners(broadcaster);
trace("");

trace("// broadcaster.addListener(listener1), again");
trace(broadcaster.addListener(listener1));
dumpListeners(broadcaster);
trace("");

trace("// broadcaster.addListener(listener2)");
trace(broadcaster.addListener(listener2));
dumpListeners(broadcaster);
trace("");

trace("// broadcaster.addListener(listener1), when not last, with tracing properties");
var listeners = [listener1, listener2];
tracingMethod("listeners", listeners, "push");
broadcaster = {addListener: AsBroadcaster.addListener, removeListener: AsBroadcaster.removeListener};
tracingMethod("broadcaster", broadcaster, "removeListener");
addTracingProperty("broadcaster", broadcaster, "_listeners", listeners);
broadcaster.__resolve = function(prop) {
    trace("  resolve broadcaster." + prop);
};
broadcaster.__proto__ = undefined;
trace(broadcaster.addListener(listener1));
dumpListeners(broadcaster);
trace("");

trace("// AsBroadcaster.addListener.call({}, listener1)");
var obj = {};
trace(AsBroadcaster.addListener.call(obj, listener1));
trace(keys(obj));
trace("");

var values = [undefined, null, 0, false, "0", NaN, NaN];
broadcaster = makeBroadcaster();
for (i = 0; i < values.length; i++) {
    var value = values[i];
    trace("// broadcaster.addListener(" + literal(value) + ")");
    trace(broadcaster.addListener(value));
    dumpListeners(broadcaster);
    trace("");
}
trace("");

trace("/// removeListener");

broadcaster = makeBroadcaster();
broadcaster._listeners = [listener1, listener2];

trace("// broadcaster.removeListener(listener2)");
trace(broadcaster.removeListener(listener2));
dumpListeners(broadcaster);
trace("");

trace("// broadcaster.removeListener(listener2), again");
trace(broadcaster.removeListener(listener2));
dumpListeners(broadcaster);
trace("");

trace("// broadcaster.removeListener(listener1), when registered twice");
broadcaster._listeners = [listener1, listener2, listener1];
trace(broadcaster.removeListener(listener1));
dumpListeners(broadcaster);
trace("");

trace("// broadcaster.removeListener(false)");
broadcaster._listeners = [null, false];
trace(broadcaster.removeListener(false));
dumpListeners(broadcaster);
trace("");

trace("// broadcaster.removeListener(undefined)");
broadcaster._listeners = [null, false];
trace(broadcaster.removeListener(undefined));
dumpListeners(broadcaster);
trace("");

trace("// broadcaster.removeListener(tracingNumber('listener', 5))");
broadcaster._listeners = [5];
trace(broadcaster.removeListener(tracingNumber("listener", 5)));
dumpListeners(broadcaster);
trace("");

trace("// broadcaster.removeListener(listener2), with tracing properties");
listeners = {};
addTracingProperty("listeners", listeners, "length", 2);
addTracingProperty("listeners", listeners, "0", listener1);
addTracingProperty("listeners", listeners, "1", listener2);
tracingMethod("listeners", listeners, "splice");
listeners.__resolve = function(prop) {
    trace("  resolve listeners." + prop);
};
listeners.__proto__ = undefined;
broadcaster = {removeListener: AsBroadcaster.removeListener};
addTracingProperty("broadcaster", broadcaster, "_listeners", listeners);
broadcaster.__resolve = function(prop) {
    trace("  resolve broadcaster." + prop);
};
broadcaster.__proto__ = undefined;
trace(broadcaster.removeListener(listener2));
trace("");
trace("");

trace("/// broadcastMessage");

broadcaster = makeBroadcaster();

trace("// broadcaster.broadcastMessage('event1'), without listeners");
trace(broadcaster.broadcastMessage("event1"));
trace("");

broadcaster._listeners = [listener1, listener2];

trace("// broadcaster.broadcastMessage('event1')");
trace(broadcaster.broadcastMessage("event1"));
trace("");

trace("// broadcaster.broadcastMessage('event2', 'foo', 123)");
trace(broadcaster.broadcastMessage("event2", "foo", 123));
trace("");

trace("// broadcaster.broadcastMessage('event1'), with a throwing listener");
broadcaster._listeners = [{event1: function() {
    trace("  throw 'error'");
    throw "error";
}}, listener1];
try {
    trace(broadcaster.broadcastMessage("event1"));
} catch (error) {
    trace("  caught " + error);
}
trace("");

trace("// broadcaster.broadcastMessage('event1'), with a listener removing itself");
broadcaster._listeners = [{event1: function() {
    trace("  broadcaster.removeListener(this)");
    broadcaster.removeListener(this);
}}, listener1];
trace(broadcaster.broadcastMessage("event1"));
dumpListeners(broadcaster);
trace("");

trace("// broadcaster.broadcastMessage('event1'), with a listener removing a later listener");
broadcaster._listeners = [{event1: function() {
    trace("  broadcaster.removeListener(listener1)");
    broadcaster.removeListener(listener1);
}}, listener1];
trace(broadcaster.broadcastMessage("event1"));
dumpListeners(broadcaster);
trace("");

trace("// broadcaster.broadcastMessage('event1'), with _listeners = undefined");
broadcaster._listeners = undefined;
trace(broadcaster.broadcastMessage("event1"));
trace("");

trace("// broadcaster.broadcastMessage('event1'), with _listeners = 'foo', with String.prototype[0] = listener1");
String.prototype[0] = listener1;
broadcaster._listeners = "foo";
trace(broadcaster.broadcastMessage("event1"));
delete String.prototype[0];
trace("");

trace("// broadcaster.broadcastMessage('event1'), with _listeners = {0: listener1}");
broadcaster._listeners = {};
broadcaster._listeners[0] = listener1;
trace(broadcaster.broadcastMessage("event1"));
trace("");

trace("// broadcaster.broadcastMessage('event1'), with _listeners = [listener1], not inheriting from Array.prototype");
broadcaster._listeners = [listener1];
broadcaster._listeners.__proto__ = Object.prototype;
trace(broadcaster.broadcastMessage("event1"));
trace("");

trace("// broadcaster.broadcastMessage('event1'), with _listeners = {0: listener1, length: 1}, inheriting from Array.prototype");
broadcaster._listeners = {};
broadcaster._listeners[0] = listener1;
broadcaster._listeners.length = 1;
broadcaster._listeners.__proto__ = Array.prototype;
trace(broadcaster.broadcastMessage("event1"));
trace("");

trace("// broadcaster.broadcastMessage('event1'), with _listeners = {0: listener1, 1: listener2} and tracing length");
broadcaster._listeners = {};
broadcaster._listeners[0] = listener1;
broadcaster._listeners[1] = listener2;
addTracingProperty("_listeners", broadcaster._listeners, "length", 1);
trace(broadcaster.broadcastMessage("event1"));
trace("");

trace("// broadcaster.broadcastMessage('event1'), with tracing properties");
var listener = {};
listener.__resolve = function(prop) {
    trace("  resolve listener." + prop);
};
listener.__proto__ = undefined;
var tracingBroadcaster = {broadcastMessage: AsBroadcaster.broadcastMessage};
addTracingProperty("broadcaster", tracingBroadcaster, "_listeners", [listener, listener1]);
tracingBroadcaster.__resolve = function(prop) {
    trace("  resolve broadcaster." + prop);
};
tracingBroadcaster.__proto__ = undefined;
trace(tracingBroadcaster.broadcastMessage("event1"));
trace("");

trace("// broadcaster.broadcastMessage('event1'), with _listeners = [, , listener1]");
broadcaster._listeners = [];
broadcaster._listeners[2] = listener1;
trace(broadcaster.broadcastMessage("event1"));
trace("");

trace("// broadcaster.broadcastMessage('event1'), with _listeners = [null, undefined]");
broadcaster._listeners = [null, undefined];
trace(broadcaster.broadcastMessage("event1"));
trace("");

trace("// broadcaster.broadcastMessage('event1'), with _listeners = [{}]");
broadcaster._listeners = [{}];
trace(broadcaster.broadcastMessage("event1"));
trace("");

trace("// broadcaster.broadcastMessage('event1'), with _listeners = ['foo', 123, true]");
var primitiveClasses = ["String", "Number", "Boolean"];
for (i = 0; i < primitiveClasses.length; i++) {
    var primitivePrototype = eval(primitiveClasses[i]).prototype;
    primitivePrototype.event1 = function() {
        trace("  this = " + this + ", typeof this = " + typeof this);
    };
    tracingMethod(primitiveClasses[i] + ".prototype", primitivePrototype, "event1");
}
broadcaster._listeners = ["foo", 123, true];
trace(broadcaster.broadcastMessage("event1"));
for (i = 0; i < primitiveClasses.length; i++) {
    delete eval(primitiveClasses[i]).prototype.event1;
}
trace("");

values = [123, undefined, "", {}];
var traceListener = function() {
    trace("  call traceListener(" + arguments.join(", ") + ")");
};
for (i = 0; i < values.length; i++) {
    tracingMethod("traceListener", traceListener, values[i]);
}
broadcaster._listeners = [traceListener];

trace("// broadcaster.broadcastMessage()");
trace(broadcaster.broadcastMessage());
trace("");

for (i = 0; i < values.length; i++) {
    value = values[i];
    trace("// broadcaster.broadcastMessage(" + literal(value) + ")");
    trace(broadcaster.broadcastMessage(value));
    trace("");
}

trace("// AsBroadcaster.broadcastMessage('event1')");
AsBroadcaster._listeners = [listener1];
trace(AsBroadcaster.broadcastMessage("event1"));
trace("");

trace("// ASnative(101, 12).call(broadcaster, 'event1')");
broadcaster = {_listeners: [listener1]};
trace(ASnative(101, 12).call(broadcaster, "event1"));
trace("");
