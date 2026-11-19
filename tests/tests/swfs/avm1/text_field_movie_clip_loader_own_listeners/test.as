function check(label, a, b) {
    trace("// " + label);
    trace("own: " + a.hasOwnProperty("_listeners"));
    trace("shared: " + (a._listeners == b._listeners));
    trace("same as the prototype's: " + (a._listeners == a.__proto__._listeners));
    trace("enumerable: " + a.isPropertyEnumerable("_listeners"));
    trace("length: " + a._listeners.length + ", contains itself: " + (a._listeners[0] === a));

    var listener = {};
    a.addListener(listener);
    trace("after addListener on one: " + a._listeners.length + ", other: " + b._listeners.length);

    var listeners = a._listeners;
    trace("delete: " + delete a._listeners + ", still the same array: " + (a._listeners == listeners));
}

var TextFieldClass = TextField;
check("new TextField()", new TextFieldClass(), new TextFieldClass());
check("new MovieClipLoader()", new MovieClipLoader(), new MovieClipLoader());

_root.createTextField("first", 1, 0, 0, 10, 10);
_root.createTextField("second", 2, 0, 20, 10, 10);
check("createTextField()", _root.first, _root.second);
