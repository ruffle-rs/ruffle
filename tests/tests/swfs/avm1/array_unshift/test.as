function listKeys(a) {
    var l = "  ";
    for (var k in a) {
        l += k + ",";
    }
    trace(l + ";");
}

function test(a) {
    trace("Before: " + a + ";");
    listKeys(a);

    var b = a.unshift(42, 43, 44);

    trace("After: " + a + ";");
    listKeys(a);

    trace("Returned: " + b + ";");
}

var a = [];
test(a);

var a = [];
a[4] = 4;
test(a);

a = [1,2,3];
test(a);

a = [];
a[2] = 2;
a[3] = 3;
a[5] = 5;
test(a);

for (var flags = 1; flags <= 7; ++flags) {
    for (var el = 0; el < 3; ++el) {
        a = [1,2,3];
        trace("ASSetPropFlags " + flags + " " + el);
        ASSetPropFlags(a, "" + el, flags, 0);
        test(a);
    }
}
