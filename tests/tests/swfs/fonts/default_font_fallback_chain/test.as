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

// Default font fallback with requested styles.
function measureStyled(text, fontName, bold, italic) {
    _root.createTextField("styledField", 2, 0, 0, 200, 50);

    var field = _root.styledField;
    field.embedFonts = false;
    field.autoSize = "left";

    var format = new TextFormat();
    format.font = fontName;
    format.size = 10;
    format.bold = bold;
    format.italic = italic;

    field.setNewTextFormat(format);
    field.text = text;

    var width = field.textWidth;
    field.removeTextField();

    return width;
}

trace("sansBoldA: " + measureStyled("a", "_sans", true, false));
trace("sansBoldGlyph: " + measureStyled(ch, "_sans", true, false));
trace("sansBoldMixed: " + measureStyled("a" + ch, "_sans", true, false));

trace("sansItalicA: " + measureStyled("a", "_sans", false, true));
trace("sansItalicGlyph: " + measureStyled(ch, "_sans", false, true));
trace("sansItalicMixed: " + measureStyled("a" + ch, "_sans", false, true));
