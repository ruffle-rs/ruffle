this.createTextField("field", 1, 10, 10, 380, 40);
field.html = true;

var fmt = new TextFormat();
fmt.size = 30;

function reset(text) {
    if (text == undefined) {
        field.htmlText = "<font size=\"12\">AB</font><font size=\"14\">CD</font>";
    } else {
        field.text = text;
    }
}

function printSizes() {
    var sizes = [];
    for (var i = 0; i < field.length; ++i) {
        sizes.push(field.getTextFormat(i).size);
    }
    trace("  text: " + field.text);
    trace("  sizes: " + sizes.join(","));
    trace("  new text format size: " + field.getNewTextFormat().size);
}

function testIndex(index) {
    reset();
    trace("setTextFormat(" + index + ", fmt)");
    field.setTextFormat(index, fmt);
    printSizes();
}

function testRange(begin, end) {
    reset();
    trace("setTextFormat(" + begin + ", " + end + ", fmt)");
    field.setTextFormat(begin, end, fmt);
    printSizes();
}

function noisy(name, value) {
    var o = new Object();
    o.valueOf = function() {
        trace("  valueOf " + name);
        return value;
    };
    o.toString = function() {
        trace("  toString " + name);
        return String(value);
    };
    return o;
}

function throwing(name, action) {
    var o = new Object();
    o.valueOf = function() {
        trace("  valueOf " + name);
        if (action != undefined) {
            action();
        }
        throw "error " + name;
    };
    o.toString = function() {
        trace("  toString " + name);
        throw "error " + name;
    };
    return o;
}

function testArgs(label, args) {
    reset();
    trace(label);
    try {
        field.setTextFormat.apply(field, args);
    } catch (e) {
        trace("  caught: " + e);
    }
    printSizes();
}

trace("initial");
reset();
printSizes();

testIndex(-1);
testIndex(0);
testIndex(2);
testIndex(3);
testIndex(4);
testIndex(5);
testIndex(100);
testIndex(1.5);
testIndex("2");
testIndex(0 / 0);
testIndex(undefined);
testIndex(4294967298);
testIndex(2147483647);

testRange(0, 4);
testRange(1, 3);
testRange(2, 2);
testRange(3, 1);
testRange(2, 4);
testRange(2, 10);
testRange(3, 4);
testRange(4, 4);
testRange(4, 5);
testRange(4, 10);
testRange(5, 10);
testRange(10, 2);
testRange(-1, 2);
testRange(0, -1);
testRange(-5, -1);
testRange(0 / 0, 0 / 0);
testRange(4294967297, 4294967299);
testRange(-4294967295, 2);

testArgs("setTextFormat(noisy 1, noisy 3, fmt)", [noisy("begin", 1), noisy("end", 3), fmt]);
testArgs("setTextFormat(noisy 2, fmt)", [noisy("index", 2), fmt]);
testArgs("setTextFormat(noisy \"1\", noisy \"3\", fmt)", [noisy("begin", "1"), noisy("end", "3"), fmt]);
testArgs("setTextFormat(noisy 1, noisy 3, {})", [noisy("begin", 1), noisy("end", 3), {}]);
testArgs("setTextFormat(noisy 2, undefined)", [noisy("index", 2), undefined]);
testArgs("setTextFormat(noisy 0, noisy 2, fmt, noisy 3)", [noisy("begin", 0), noisy("end", 2), fmt, noisy("extra", 3)]);
testArgs("setTextFormat(noisy 0, noisy 2, noisy 3, fmt)", [noisy("a", 0), noisy("b", 2), noisy("c", 3), fmt]);
testArgs("setTextFormat(throwing, noisy 3, fmt)", [throwing("begin"), noisy("end", 3), fmt]);
testArgs("setTextFormat(noisy 1, throwing, fmt)", [noisy("begin", 1), throwing("end"), fmt]);
testArgs("setTextFormat(throwing, fmt)", [throwing("index"), fmt]);
testArgs("setTextFormat(throwing, throwing, {})", [throwing("begin"), throwing("end"), {}]);
testArgs("setTextFormat(throwing, {})", [throwing("index"), {}]);
testArgs("setTextFormat(throwing replacing text, fmt)", [throwing("index", function() {
    field.text = "abcdefgh";
}), fmt]);
var fmt2 = new TextFormat();
fmt2.size = 20;
testArgs("setTextFormat(throwing changing fmt2.size, fmt2)", [throwing("index", function() {
    fmt2.size = 50;
}), fmt2]);

trace("empty text, setTextFormat(0, fmt)");
reset("");
field.setTextFormat(0, fmt);
printSizes();
reset("ABCD");
printSizes();

trace("empty text, setTextFormat(0, 1, fmt)");
reset("");
field.setTextFormat(0, 1, fmt);
printSizes();
reset("ABCD");
printSizes();

trace("empty text, setTextFormat(fmt)");
reset("");
field.setTextFormat(fmt);
printSizes();
reset("ABCD");
printSizes();
