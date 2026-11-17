function testDecls(object) {
    var enumerated = {};
    for (var key in object) {
        enumerated[key] = true;
    }

    ASSetPropFlags(object, null, 0, 1);
    var hasOwnProperty = Object.prototype.hasOwnProperty;
    for (var key in object) {
        var properties = "";
        var isOwn = hasOwnProperty.call(object, key);
        if (isOwn) {
            properties += ", own";
        }

        var old = object[key];

        if (!enumerated[key]) {
            properties += ", DONT_ENUM";
        }

        object[key] = "OTHER";
        if (object[key] !== "OTHER") {
            properties += ", READ_ONLY";
        }
        object[key] = old;

        var t = typeof(old);
        if (t === "function" && old.prototype) {
            t = "function with prototype";
        }

        properties += ", type=[" + t + "]";

        trace("  " + key + properties);
    }
}

function callAndRecurse(prop, name) {
    // Don't recurse infinitely.
    if (prop === "__proto__") return;
    if (prop === "constructor") return;

    // i is the object, name is its name.
    name = (name == "" ? "_global" : name + "." + prop);
    var i = eval(name);

    trace("Testing " + name);
    testDecls(i);

    if (i && i.prototype) {
        trace("Testing " + name + ".prototype");
        testDecls(i.prototype);
    }

    if (typeof(i) == "object") {
        ASSetPropFlags(i, null, 0, 1);
        for (p in i) {
            callAndRecurse(p, name);
        }
    }
};

// Skip this, it is only present in the debugger version of FP.
delete _global.enableDebugConsole;

callAndRecurse(_global, "");

trace("Done");
fscommand("quit");
