// Device font lookup by name: case-insensitivity, and that prefixes,
// suffixes, and spaces do not cause false matches.
//
// TestFontA and TestFontB are provided as device fonts (see test.toml).
// They render "abc" with distinct widths, so comparing the rendered width
// against each base font's width tells us which font (if any) a name
// resolved to. A device font name that doesn't match either falls back to
// the default device font, whose width equals neither.

function widthFor(fontName) {
    var tf = _root.createTextField("tf", _root.getNextHighestDepth(), 0, 0, 200, 50);
    tf.embedFonts = false; // device font
    tf.setNewTextFormat(new TextFormat(fontName, 20));
    tf.text = "abc";
    var w = tf.textWidth;
    tf.removeTextField();
    return w;
}

var wA = widthFor("TestFontA");
var wB = widthFor("TestFontB");

function check(label, fontName) {
    var w = widthFor(fontName);
    trace(label + " -> A=" + (w == wA) + ", B=" + (w == wB));
}

// Case-insensitive: these should resolve to TestFontA.
check("lower testfonta", "testfonta");
check("upper TESTFONTA", "TESTFONTA");
check("mixed tEsTfOnTa", "tEsTfOnTa");

// Case-insensitive: these should resolve to TestFontB.
check("lower testfontb", "testfontb");
check("upper TESTFONTB", "TESTFONTB");

// Prefix / suffix / space: these should resolve to neither (fallback).
check("prefix XTestFontA", "XTestFontA");
check("suffix TestFontAX", "TestFontAX");
check("space Test FontA", "Test FontA");
check("leading space  TestFontA", " TestFontA");
check("trailing space TestFontA ", "TestFontA ");
check("suffix-hyphen TestFontA-Bold", "TestFontA-Bold");

trace("done");
