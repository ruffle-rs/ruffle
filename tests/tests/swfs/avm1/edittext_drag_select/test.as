var listener = new Object();
listener.onMouseDown = function() {
    trace("Mouse down: " + _root._xmouse + "," + _root._ymouse)
};
listener.onMouseUp = function() {
    trace("Mouse up: " + _root._xmouse + "," + _root._ymouse)
    text.replaceSel("<selection>");
    trace("Text: " + text.text.split("\r").join("\\n").split("\n").join("\\n"));
};
Key.addListener(listener);
Mouse.addListener(listener);
