var nc = new NetConnection();
nc.connect("http://localhost:8000/");

// Callback function
var responder = new Object();
responder.onResult = function(result) {
    trace("Received result");
};

// 1. Genuine Array
var realArray = new Array();
realArray[0] = "real_0";
realArray[1] = "real_1";

// 2. Fake Array
var fakeArray = new Object();
fakeArray[0] = "fake_0";
fakeArray[1] = "fake_1";
fakeArray.length = 2;

// 3. Mixed Array (Genuine Array with a custom string property)
var mixedArray = new Array();
mixedArray["a_prop"] = "value_a";
mixedArray[0] = "mixed_0";
mixedArray["m_prop"] = "value_m";
mixedArray[1] = "mixed_1";
mixedArray["z_prop"] = "value_z";
mixedArray["b_prop"] = "value_b";

// 4. Sparse Array with "Holes" (Should remain a StrictArray for NetConnection)
var sparseArray = new Array();
sparseArray[0] = "sparse_index_0";
sparseArray[5] = "sparse_index_5"; // Leaving indices 1, 2, 3, 4 undefined

// 5. Deeply Nested Dense Array
var nestedContainer = {
    someProperty: "hello",
    deepArray: ["nested_0", "nested_1"]
};

function arrayElementFunction() {
    return "function_result";
}

// 6. Top-level function-valued array. NetConnection serializes this as a
// StrictArray, exercising serialize_array's function-element handling.
var functionArray = new Array();
functionArray[0] = arrayElementFunction;
functionArray[1] = "function_keep";

// 7. Function-valued array nested in an ordinary object. This takes the
// ObjWriter/strict_array path in serialize_value_to_writer.
var nestedFunctionArray = new Array();
nestedFunctionArray[0] = arrayElementFunction;
nestedFunctionArray[1] = "nested_function_keep";
var nestedFunctionContainer = {
    label: "nested function array",
    values: nestedFunctionArray
};

trace("--- Testing NetConnection Array Serialization ---");
nc.call(
    "test.arrays",
    responder,
    realArray,
    fakeArray,
    mixedArray,
    sparseArray,
    nestedContainer,
    functionArray,
    nestedFunctionContainer
);
