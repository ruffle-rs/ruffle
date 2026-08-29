// Regression test for #23453.
// `restrict` limits which characters the user is allowed to type, but it must
// not stop the Enter key from starting a new line in a multiline input field.
// Here "a-z" disallows the newline character, yet Enter should still insert it.
//
// The text is reported as character codes so that the expected output stays
// readable and does not itself contain a line break.

this.createTextField("field", 1, 0, 0, 200, 100);
field.type = "input";
field.multiline = true;
field.restrict = "a-z";

Selection.setFocus(field);

var listener = new Object();
listener.onKeyDown = function() {
    if (Key.getCode() == 27) {
        var text = field.text;
        trace("Length: " + text.length);
        for (var i = 0; i < text.length; i++) {
            trace("Char " + i + ": " + text.charCodeAt(i));
        }
    }
};
Key.addListener(listener);
