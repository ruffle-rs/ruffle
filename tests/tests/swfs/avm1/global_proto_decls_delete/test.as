function testDecls(object) {
    ASSetPropFlags(object, null, 0, 1);
    for (var key in object) {
        var properties = "";

        var old = object[key];

        delete object[key];
        var enumeratedAgain = false;
        for (var key2 in object) {
            if (key === key2) {
                enumeratedAgain = true;
                break;
            }
        }

        if (enumeratedAgain) {
            trace("  " + key + ", DONT_DELETE");
        }
    }
}

function callAndRecurse(prop, name) {
    // Don't recurse infinitely.
    if (prop == "__proto__") return;
    if (prop == "constructor") return;

    // i is the object, name is its name.
    name = name + "." + prop;
    var i = eval(name);

    if (typeof(i) == "object") {
        ASSetPropFlags(i, null, 0, 1);
        for (p in i) {
            callAndRecurse(p, name);
        }
    }

    trace("Testing " + name);
    testDecls(i);

    if (i.prototype) {
        trace("Testing " + name + ".prototype");
        testDecls(i.prototype);
    }
};

start = "_global";
obj = eval(start);

ASSetPropFlags(obj, null, 0, 1);
for (prop in obj) {
    callAndRecurse(prop, start);
}

trace("Done");
