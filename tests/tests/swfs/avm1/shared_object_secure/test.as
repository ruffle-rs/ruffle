var noisy = {
    valueOf: function() {
        trace("valueOf called");
        return 0;
    },
    toString: function() {
        trace("toString called");
        return "false";
    }
};
var throwingValueOf = {
    valueOf: function() {
        throw "valueOf error";
    }
};
var throwingToString = {
    toString: function() {
        throw "toString error";
    }
};
var values = [
    "omitted", undefined,
    "true", true,
    "false", false,
    "\"true\"", "true",
    "\"false\"", "false",
    "\"1\"", "1",
    "\"0\"", "0",
    "\"abc\"", "abc",
    "\"\"", "",
    "1", 1,
    "0", 0,
    "-1", -1,
    "NaN", NaN,
    "Infinity", Infinity,
    "null", null,
    "undefined", undefined,
    "object", {},
    "noisy object", noisy,
    "throwing valueOf", throwingValueOf,
    "throwing toString", throwingToString
];
for (var i = 0; i < values.length; i += 2) {
    var so;
    try {
        if (i == 0) {
            so = SharedObject.getLocal("so" + i, null);
        } else {
            so = SharedObject.getLocal("so" + i, null, values[i + 1]);
        }
    } catch (e) {
        trace(values[i] + ": caught " + e);
        continue;
    }
    trace(values[i] + ": " + (so == null ? "null" : "created"));
}
