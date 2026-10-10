function Error(message) {
    if (message !== undefined) {
        this.message = message;
    }
}

var o = Error.prototype;

// message is assigned before name, which affects property enumeration order.
o.name = o.message = "Error";

o.toString = function() {
    return this.message;
};

function AsSetupError(names) {
    // Dispatch through `names.split`, which can be overridden.
    var namesArray = names.split(",");
    for (var i = 0; i < namesArray.length; i++) {
        var name = namesArray[i];
        // Construct through global `Error`, which can be overridden.
        var errorProto = new Error();
        // name is assigned before message, which is observable via watchers and setters.
        errorProto.name = name;
        errorProto.message = name;
        // errorProto is not stored anyway, and so unreachable.
    }
}
