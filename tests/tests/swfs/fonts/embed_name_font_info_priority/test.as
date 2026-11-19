function check(fontName, html) {
    var name = "tf" + _root.getNextHighestDepth();
    _root.createTextField(name, _root.getNextHighestDepth(), 0, 0, 200, 100);
    var t = _root[name];
    t.embedFonts = true;
    if (html) {
        t.html = true;
        t.htmlText = "<font face=\"" + fontName + "\" size=\"20\">0123</font>";
    } else {
        t.setNewTextFormat(new TextFormat(fontName, 20));
        t.text = "0123";
    }
    trace(fontName + (html ? " (html)" : "") + ": " + t.textWidth);
}

check("TestFont", false);
check("TestFont", true);
check("TestFont2", false);
check("TestFont2", true);
check("InfoOnly", false);
check("InfoOnly", true);
check("Italic2", false);
check("Bold2", false);
check("BoldItalic2", false);
check("Italic1", false);
