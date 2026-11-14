this.createTextField("field", 1, 10, 10, 380, 40);
field.html = true;

var fmt = new TextFormat();
fmt.size = 30;

// Sizes 20-27.
var longText = "";
for (var i = 0; i < 8; ++i) {
    longText += "<font size=\"" + (20 + i) + "\">" + "abcdefgh".charAt(i) + "</font>";
}

function setLong() {
    field.htmlText = longText;
}

function setShort() {
    field.htmlText = "<font size=\"20\">x</font><font size=\"21\">y</font>";
}

function noisy(name, value, action) {
    var o = new Object();
    o.valueOf = function() {
        trace("  valueOf " + name);
        if (action != undefined) {
            action();
        }
        return value;
    };
    return o;
}

function testArgs(label, args) {
    field.htmlText = "<font size=\"12\">AB</font><font size=\"14\">CD</font>";
    trace("setTextFormat(" + label + ")");
    field.setTextFormat.apply(field, args);
    var sizes = [];
    for (var i = 0; i < field.length; ++i) {
        sizes.push(field.getTextFormat(i).size);
    }
    trace("  text: " + field.text);
    trace("  sizes: " + sizes.join(","));
}

testArgs("noisy 6 setting long text, fmt", [noisy("index", 6, setLong), fmt]);
testArgs("noisy 3 setting short text, fmt", [noisy("index", 3, setShort), fmt]);
testArgs("noisy 1 setting short text, fmt", [noisy("index", 1, setShort), fmt]);
testArgs("noisy 1, noisy 7 setting long text, fmt", [noisy("begin", 1), noisy("end", 7, setLong), fmt]);
testArgs("noisy 5 setting long text, noisy 7, fmt", [noisy("begin", 5, setLong), noisy("end", 7), fmt]);
testArgs("noisy 0, noisy 4 setting short text, fmt", [noisy("begin", 0), noisy("end", 4, setShort), fmt]);
testArgs("noisy 3 setting short text, noisy 4, fmt", [noisy("begin", 3, setShort), noisy("end", 4), fmt]);
testArgs("noisy 1 setting long text, noisy 4 setting short text, fmt", [noisy("begin", 1, setLong), noisy("end", 4, setShort), fmt]);
