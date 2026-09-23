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

trace("primaryA: " + measureText("a", "TestFontA"));
trace("secondaryB: " + measureText("b", "TestFontB"));
trace("sansA: " + measureText("a", "_sans"));
trace("sansB: " + measureText("b", "_sans"));
