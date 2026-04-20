function testDecls(object) {
    var enumerated = {};
    for (var key in object) {
        enumerated[key] = true;
    }

    ASSetPropFlags(object, null, 0, 1);
    for (var key in object) {
        var properties = "";
        var isOwn = object.hasOwnProperty(key);
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

        properties += ", type=[" + (typeof(old)) + "]";

        trace("  " + key + properties);
    }
}

function callAndRecurse(prop, name) {
    // Don't recurse infinitely.
    if (prop == "__proto__") return;
    if (prop == "constructor") return;

    // i is the object, name is its name.
    name = name + "." + prop;
    var i = eval(name);

    trace("Testing " + name);
    testDecls(i);

    if (i.prototype) {
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

start = "_global";
obj = eval(start);

ASSetPropFlags(obj, null, 0, 1);
for (prop in obj) {
    callAndRecurse(prop, start);
}

trace("Done");
