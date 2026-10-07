function measureText(text, fontName) {
    _root.createTextField("field", 1, 0, 0, 200, 50);

    var field = _root.field;
    field.embedFonts = false;
    field.autoSize = "left";

    var format = new TextFormat();
    format.font = fontName;
    format.size = 10;

    field.setNewTextFormat(format);
    field.text = text;

    var width = field.textWidth;

    field.removeTextField();

    return width;
}


var ch = String.fromCharCode(0x0416);
trace("primaryA: " + measureText("a", "TestFontA"));
trace("secondaryGlyph: " + measureText(ch, "TestFontB"));
trace("sansA: " + measureText("a", "_sans"));
trace("sansGlyph: " + measureText(ch, "_sans"));
trace("sansMixed: " + measureText("a" + ch, "_sans"));
