var listener = new Object();
listener.onMouseUp = function(event) {
    field.replaceSel("|");
    trace("    { \"type\": \"MouseMove\", \"pos\": [" + _xmouse + ", " + _ymouse + "] },");
    trace("    { \"type\": \"MouseDown\", \"pos\": [" + _xmouse + ", " + _ymouse + "], \"btn\": \"Left\" },");
    trace("    { \"type\": \"MouseUp\", \"pos\": [" + _xmouse + ", " + _ymouse + "], \"btn\": \"Left\" },");
    trace("Text with caret: " + field.text.split("\r").join("\\n").split("\n").join("\\n"));
    field.text = "This is an example text\nand this is its second line";
};
Mouse.addListener(listener);
Selection.setFocus(field);
