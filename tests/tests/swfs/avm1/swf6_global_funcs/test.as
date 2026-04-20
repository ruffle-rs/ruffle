var funcs = [Boolean, isNaN, isFinite, Number];
var values = [null, undefined, true, false, "true", "false", "0", "1", "-5", "True", "False", "Infinity", "-Infinity", "NaN", "", " ", "Hello", " 5", "  5  ", "inf", "-inf", "100a", "0xhello", "123e-1", "0xUIXUIDFKHJDF012345678", "0x12", "0x10", "0x1999999981ffffff", "037777777777", "-010", "+010", " -010", " +010", " 010", "037777777777", "-037777777777", "Hello", "\t\r12", "\r12\r", " \t 12", -NaN, NaN, -Infinity, Infinity, 1, 2, 3, 0, -0.1, -0.5, 0.1, 0.5, -1, -2, -0.9, 0.9, {}];
for (var i in funcs) {
    var func = funcs[i];
    for (var j in values) {
        var value = values[j];
        trace(func(value));
    }
    trace(func());
}
