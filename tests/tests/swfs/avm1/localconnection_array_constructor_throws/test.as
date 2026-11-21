// LocalConnection should not fail the entire message when the script-visible
// Array constructor throws while rebuilding an AMF0 ECMAArray or StrictArray.
trace("--- Starting LocalConnection Array constructor error test ---");

var savedArrayConstructor = _global.Array;
var throwingArrayConstructorCalls = 0;

function ThrowingArray() {
    throwingArrayConstructorCalls++;
    throw "intentional Array constructor error";
}

var receiver = new LocalConnection();
receiver.onReceiveArrays = function(ecmaValue, strictValue) {
    trace("--- Received LocalConnection arrays ---");
    trace("ECMAArray result type: " + typeof(ecmaValue));
    trace("StrictArray result type: " + typeof(strictValue));
    trace("Throwing Array constructor calls: " + throwingArrayConstructorCalls);

    // Restore the global after both arguments have been deserialized.
    _global.Array = savedArrayConstructor;
    trace("Restored Array constructor: " + (_global.Array == savedArrayConstructor));
};
trace("Connected: " + receiver.connect("array_constructor_throws"));

var sender = new LocalConnection();

// An explicit undefined element forces LocalConnection to use an ECMAArray.
var ecmaArray = new Array();
ecmaArray[0] = undefined;
ecmaArray[1] = "ecma-tail";

// A dense array with no undefined/function/display-object values uses a
// StrictArray.
var strictArray = ["strict-first", "strict-second"];

trace("Original Array constructor is callable: " + (typeof(savedArrayConstructor) == "function"));
_global.Array = ThrowingArray;
sender.send(
    "array_constructor_throws",
    "onReceiveArrays",
    ecmaArray,
    strictArray
);
