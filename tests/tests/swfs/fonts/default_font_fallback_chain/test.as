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

var primaryA = measureText("a", "TestFontA");
var secondaryB = measureText("b", "TestFontB");

var sansA = measureText("a", "_sans");
var sansB = measureText("b", "_sans");

trace("primary font works: " + (primaryA > 0));
trace("secondary font works: " + (secondaryB > 0));
trace("sans uses primary font: " + (sansA == primaryA));
trace("sans uses secondary fallback: " + (sansB == secondaryB));
