function testSelection() {
    trace("Focus: " + Selection.getFocus());
    var before = text.text;
    text.replaceSel("|");
    trace("Selection: " + before + " -> " + text.text);
    text.text = "text";
}

testSelection();
Selection.setFocus(text);
testSelection();
Selection.setFocus(text);
testSelection();
testSelection();
testSelection();
Selection.setFocus(text);
testSelection();
