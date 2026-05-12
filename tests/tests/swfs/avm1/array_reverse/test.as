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

    var b = a.reverse();

    trace("After: " + a + ";");
    listKeys(a);

    trace("Returned: " + b + ";");
    listKeys(b);
}

var a = [];
test(a);

var a = [];
a[6] = 4;
test(a);

a = [];
a[3] = 3;
test(a);

a = [1,2,3];
test(a);

a = [];
a[2] = 2;
a[3] = 3;
a[5] = 5;
test(a);

for (var i = 1; i <= 7; ++i) {
    for (var el = 0; el < 4; ++el) {
        a = [1,2,3,4,5];
        trace("ASSetPropFlags " + i + " " + el);
        ASSetPropFlags(a, "" + el, i, 0);
        test(a);
    }
}
