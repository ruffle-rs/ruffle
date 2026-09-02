var alpha = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"];
var fontNames = [];

function checkFont(fontName, isBold) {
    var t = _root.createTextField("tf" + fontName, _root.getNextHighestDepth(), 0, 0, 100, 100);
    t.embedFonts = true;
    var tf = new TextFormat(fontName, 10);
    if (isBold === true) {
        tf.bold = true;
    }
    t.setNewTextFormat(tf);
    t.text = "abcd";

    trace(fontName + (tf.bold ? " (bold)" : "") + ": " + t.textWidth);
}

function checkFonts() {
    for (var i = 0; i < alpha.length; ++i) {
        for (var j = 0; j < alpha.length; ++j) {
            var fontName = "" + alpha[i] + alpha[j];
            checkFont(fontName);
            checkFont(fontName.toLowerCase());
            checkFont(fontName, true);
            if (fontName == "CZ") return;
        }
    }
}

checkFonts();
