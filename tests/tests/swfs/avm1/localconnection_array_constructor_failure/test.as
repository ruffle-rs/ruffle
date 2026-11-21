// LocalConnection uses the script-visible global Array constructor while
// rebuilding AMF0 ECMAArray and StrictArray arguments. Make that binding
// unavailable for the receive operation to exercise both failure arms.
var savedArrayConstructor = _global.Array;

var receiver = new LocalConnection();
receiver.onReceiveArrays = function(ecmaValue, strictValue) {
    trace("ECMAArray result type: " + typeof(ecmaValue));
    trace("StrictArray result type: " + typeof(strictValue));

    // Restore the global as soon as the arguments have been deserialized.
    _global.Array = savedArrayConstructor;
    trace("Restored Array constructor: " + (typeof(_global.Array) == "function"));
};
trace("Connected: " + receiver.connect("array_constructor_failure"));

var sender = new LocalConnection();

// LocalConnection encodes an array with an explicit undefined entry as an
// ECMAArray, preserving that entry rather than treating it as a hole.
var ecmaArray = new Array();
ecmaArray[0] = undefined;
ecmaArray[1] = "ecma-tail";

// A dense array with no undefined/function/display-object values is encoded
// as a StrictArray.
var strictArray = ["strict-first", "strict-second"];

trace("Original Array constructor: " + (typeof(savedArrayConstructor) == "function"));
_global.Array = undefined;
sender.send(
    "array_constructor_failure",
    "onReceiveArrays",
    ecmaArray,
    strictArray
);
