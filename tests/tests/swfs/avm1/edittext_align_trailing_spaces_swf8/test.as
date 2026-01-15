function test3(text, autoSize, align) {
    _root.createTextField("t", _root.getNextHighestDepth(), 0, 0, 1, 1);
    t.embedFonts = true;

    var tf = new TextFormat("TestFont", 20);
    tf.align = align;
    t.setNewTextFormat(tf);
    t.autoSize = autoSize;
    t.text = text;

    trace(autoSize + ", " + align + " \"" + text + "\"");
    trace("  x, y: " + t._x + ", " + t._y);
    trace("  width, height: " + t._width + ", " + t._height);
    trace("  textWidth: " + t.textWidth);

    t.removeTextField();
}

function test2(text, autoSize) {
    test3(text, autoSize, "left");
    test3(text, autoSize, "right");
    test3(text, autoSize, "center");
}

function test(text) {
    test2(text, "none");
    test2(text, "left");
    test2(text, "right");
    test2(text, "center");
}

test("");
test(" ");
test("  ");
test("   ");
test("aa ");
test("aa  ");
test(" aa");
test("  aa");
test(" aa ");
test("  aa ");
test(" aa  ");
test("  aa  ");
