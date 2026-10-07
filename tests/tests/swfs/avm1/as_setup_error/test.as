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
        var args = "";
        for (var i = 0; i < arguments.length; i++) {
            if (i > 0) {
                args += ", ";
            }
            args += arguments[i];
        }
        trace("  call " + name + "." + method + "(" + args + ")");
        return original.apply(this, arguments);
    };
}

function tracingNumber(name, value) {
    return {valueOf: function() {
        trace("  valueOf " + name);
        return value;
    }};
}

var originalError = Error;
var lastError;

function installTracingError() {
    _global.Error = function() {
        trace("  new Error(" + arguments.join(", ") + ")");
        addTracingProperty("error", this, "name", undefined);
        addTracingProperty("error", this, "message", undefined);
        lastError = this;
    };
}

function AsSetupErrorWithTracingError(names) {
    installTracingError();
    AsSetupError(names);
    _global.Error = originalError;
}

trace("/// Return value and global state");

trace("// AsSetupError('Foo')");
trace(AsSetupError("Foo"));
trace("// typeof Foo");
trace(typeof Foo);
trace("");
trace("");

trace("/// String.prototype.split dispatch");

var originalStringSplit = String.prototype.split;
tracingMethod("String.prototype", String.prototype, "split");

trace("// AsSetupError('Foo,Bar')");
AsSetupErrorWithTracingError("Foo,Bar");
trace("");

String.prototype.split = originalStringSplit;
trace("");

trace("/// Non-string names");

trace("// AsSetupError(12) with Number.prototype.split");
Number.prototype.split = function() {
    return ["Num"];
};
AsSetupErrorWithTracingError(12);
trace("");

trace("// AsSetupError(undefined)");
AsSetupErrorWithTracingError(undefined);
trace("");
trace("");

trace("/// Custom split");

var names = {};
names.split = function() {
    var result = {};
    addTracingProperty("result", result, "length", tracingNumber("result.length", 1));
    addTracingProperty("result", result, 0, {});
    return result;
};

trace("// AsSetupError(names)");
AsSetupErrorWithTracingError(names);
trace("// typeof error.name, typeof error.message");
trace(typeof lastError.name + ", " + typeof lastError.message);
trace("");
trace("");

trace("/// Error lookup");

trace("// AsSetupError('Foo,Bar') with Error replaced during the first construction");
_global.Error = function() {
    trace("  new FirstError()");
    installTracingError();
};
AsSetupError("Foo,Bar");
_global.Error = originalError;
trace("");

trace("// AsSetupError('Foo') with Error also defined on _root");
_root.Error = function() {
    trace("  new _root.Error()");
};
AsSetupError("Foo");
delete _root.Error;
trace("");
