var o = flash.external.ExternalInterface;

o._callIn = function() {
    // TODO: Implement.
};

o._arrayToXML = function(array) {
    var result = "<array>";
    for (var i = 0; i < array.length; i++) {
        result += "<property id=\"" + i + "\">" + flash.external.ExternalInterface._toXML(array[i]) + "</property>";
    }
    result += "</array>";
    return result;
};

o._argumentsToXML = function(args) {
    var result = "<arguments>";
    // The first argument is the function name, so it's skipped.
    for (var i = 1; i < args.length; i++) {
        result += flash.external.ExternalInterface._toXML(args[i]);
    }
    result += "</arguments>";
    return result;
};

o._objectToXML = function(object) {
    var result = "<object>";
    for (var prop in object) {
        result += "<property id=\"" + prop + "\">" + flash.external.ExternalInterface._toXML(object[prop]) + "</property>";
    }
    result += "</object>";
    return result;
};

o._toXML = function(value) {
    if (value === undefined) {
        return "<undefined/>";
    }

    if (value == null) {
        return "<null/>";
    }

    if (value === true) {
        return "<true/>";
    }

    if (value === false) {
        return "<false/>";
    }

    var type = typeof value;

    if (type === "number") {
        return "<number>" + value + "</number>";
    }

    if (type === "string") {
        return "<string>" + flash.external.ExternalInterface._escapeXML(value) + "</string>";
    }

    if (value.hasOwnProperty("length")) {
        // Yes, `new String("hello")` serializes to an array with 5 undefined elements.
        return flash.external.ExternalInterface._arrayToXML(value);
    }

    if (type === "object") {
        return flash.external.ExternalInterface._objectToXML(value);
    }

    return "<null/>";
};

o._objectToAS = function(node) {
    // TODO: Implement.
    return {};
};

o._arrayToAS = function() {
    // TODO: Implement.
    return [];
};

o._toAS = function(node) {
    var type = node.nodeName;

    if (type == "null") {
        return null;
    }

    if (type == "true") {
        return true;
    }

    if (type == "false") {
        return false;
    }

    if (type == "string") {
        return String(node.firstChild);
    }

    if (type == "number") {
        return Number(node.firstChild.toString());
    }

    if (type == "array") {
        return flash.external.ExternalInterface._arrayToAS(node);
    }

    if (type == "object") {
        return flash.external.ExternalInterface._objectToAS(node);
    }

    if (type == "arguments") {
        return flash.external.ExternalInterface._argumentsToAS(node);
    }

    if (type == "class") {
        // TODO: Implement.
        return undefined;
    }

    return undefined;
};

o._argumentsToAS = function(node) {
    // TODO: Implement.
    return [];
};

o._arrayToJS = function() {
    // TODO: Implement.
};

o._objectToJS = function() {
    // TODO: Implement.
};

o._toJS = function() {
    // TODO: Implement.
};

// DONT_ENUM, DONT_DELETE, READ_ONLY, and only visible to SWF 8 and later.
// This also covers the members defined in Rust, and `prototype`, `__proto__` and `constructor`.
ASSetPropFlags(o, null, 4103);
