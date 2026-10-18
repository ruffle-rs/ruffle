this.createTextField("field", 1, 10, 10, 380, 40);
field.html = true;
var initialText = "<b><font size=\"10\" color=\"#FF0000\">A</font></b>"
    + "<font size=\"12\" color=\"#FF0000\">B</font>"
    + "<font size=\"14\" color=\"#0000FF\">C</font>"
    + "<i><font size=\"16\" color=\"#0000FF\">D</font></i>";
field.htmlText = initialText;

// Sizes 20-27, bold from the fifth character.
var longText = "";
for (var i = 0; i < 8; ++i) {
    var ch = "<font size=\"" + (20 + i) + "\">" + "abcdefgh".charAt(i) + "</font>";
    longText += i >= 4 ? "<b>" + ch + "</b>" : ch;
}
var shortText = "<font size=\"30\">x</font><font size=\"31\">y</font>";

function noisy(name, value) {
    var o = new Object();
    o.valueOf = function() {
        trace("  valueOf " + name);
        return value;
    };
    return o;
}

function throwing(name) {
    var o = new Object();
    o.valueOf = function() {
        trace("  valueOf " + name);
        throw "error " + name;
    };
    return o;
}

function changing(name, text, value) {
    var o = new Object();
    o.valueOf = function() {
        trace("  valueOf " + name);
        field.htmlText = text;
        return value;
    };
    return o;
}

function check(label, args) {
    trace(label);
    try {
        var tf = field.getTextFormat.apply(field, args);
        trace("  size=" + tf.size + " color=" + tf.color + " bold=" + tf.bold
            + " italic=" + tf.italic + " underline=" + tf.underline);
    } catch (e) {
        trace("  caught: " + e);
    }
}

function callAll(min, max) {
    check("getTextFormat()", []);
    for (var i = min; i <= max; ++i) {
        check("getTextFormat(" + i + ")", [i]);
    }
    for (var i = min; i <= max; ++i) {
        for (var j = min; j <= max; ++j) {
            check("getTextFormat(" + i + ", " + j + ")", [i, j]);
        }
    }
}

callAll(-2, 5);

check("getTextFormat(1.5)", [1.5]);
check("getTextFormat(\"2\")", ["2"]);
check("getTextFormat(NaN)", [0 / 0]);
check("getTextFormat(undefined)", [undefined]);
check("getTextFormat(4294967298)", [4294967298]);
check("getTextFormat(2147483647)", [2147483647]);
check("getTextFormat(-4294967295)", [-4294967295]);
check("getTextFormat(1.5, 3.5)", [1.5, 3.5]);
check("getTextFormat(NaN, NaN)", [0 / 0, 0 / 0]);
check("getTextFormat(0, NaN)", [0, 0 / 0]);
check("getTextFormat(4294967297, 4294967299)", [4294967297, 4294967299]);
check("getTextFormat(-4294967295, 2)", [-4294967295, 2]);
check("getTextFormat(0, 2147483647)", [0, 2147483647]);

check("getTextFormat(noisy 1, noisy 3)", [noisy("begin", 1), noisy("end", 3)]);
check("getTextFormat(noisy 2)", [noisy("index", 2)]);
check("getTextFormat(noisy 0, noisy 2, noisy 3)", [noisy("begin", 0), noisy("end", 2), noisy("extra", 3)]);
check("getTextFormat(throwing, noisy 3)", [throwing("begin"), noisy("end", 3)]);
check("getTextFormat(noisy 1, throwing)", [noisy("begin", 1), throwing("end")]);
check("getTextFormat(throwing)", [throwing("index")]);

check("getTextFormat(changing to long 6)", [changing("index", longText, 6)]);
field.htmlText = initialText;
check("getTextFormat(changing to short 3)", [changing("index", shortText, 3)]);
field.htmlText = initialText;
check("getTextFormat(changing to short 1)", [changing("index", shortText, 1)]);
field.htmlText = initialText;
check("getTextFormat(noisy 1, changing to long 7)", [noisy("begin", 1), changing("end", longText, 7)]);
field.htmlText = initialText;
check("getTextFormat(changing to long 5, noisy 7)", [changing("begin", longText, 5), noisy("end", 7)]);
field.htmlText = initialText;
check("getTextFormat(noisy 0, changing to short 4)", [noisy("begin", 0), changing("end", shortText, 4)]);
field.htmlText = initialText;
check("getTextFormat(changing to short 3, noisy 4)", [changing("begin", shortText, 3), noisy("end", 4)]);
field.htmlText = longText;
check("getTextFormat() on long text", []);
check("getTextFormat(1, 7) on long text", [1, 7]);
check("getTextFormat(1, 4) on long text", [1, 4]);
check("getTextFormat(5, 7) on long text", [5, 7]);

trace("empty text");
field.text = "";
callAll(-2, 1);
