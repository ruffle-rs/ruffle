function checkSel() {
    var before = text.text;
    text.replaceSel("|");
    var after = text.text;
    trace(before + "; " + after);
}

_root.createTextField(
    "text",
    _root.getNextHighestDepth(),
    10, 10, 100, 30);

text.text = "abcd";
checkSel();

Selection.setFocus(text);
checkSel();
