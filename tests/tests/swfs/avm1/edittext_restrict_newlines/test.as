function escapeNewlines(text) {
    return text
        .split("\r").join("\\r")
        .split("\n").join("\\n");
}

function createInputTextField(name, y, multiline) {
    _root.createTextField(name, _root.getNextHighestDepth(), 0, y, 200, 40);

    var field = _root[name];
    field.type = "input";
    field.multiline = multiline;
    field.restrict = "A-Za-z0-9";
    field.text = "";

    return field;
}

function traceField(name, field) {
    trace(name + ".multiline: " + field.multiline);
    trace(name + ".text: " + escapeNewlines(field.text));
}

var singleLine = createInputTextField("singleLine", 0, false);
var multiLine = createInputTextField("multiLine", 50, true);
var currentField = 0;

var listener = new Object();
listener.onKeyDown = function() {
    if (Key.getCode() == 27) {
        if (currentField == 0) {
            traceField("singleLine", singleLine);
            Selection.setFocus(multiLine);
            currentField = 1;
        } else {
            traceField("multiLine", multiLine);
        }
    }
};
Key.addListener(listener);

Selection.setFocus(singleLine);
