// --- 1. SETUP THE DATA STRUCTURES ---

var denseArray = new Array();
denseArray.push("dense_0", "dense_1");

var sparseArray = new Array();
sparseArray[0] = "sparse_0";
sparseArray[5] = "sparse_5";

var mixedArray = new Array();
mixedArray.push("mixed_0");
mixedArray["custom_prop"] = "custom_value";

var fakeArray = new Object();
fakeArray["0"] = "fake_0";
fakeArray["length"] = 1;

// --- 2. TEST NETCONNECTION (AMF0 Wire Serialization) ---
trace("--- Testing NetConnection ---");
var nc = new NetConnection();
nc.connect("http://localhost:8000/");

var responder = new Object();
responder.onResult = function(res) {
  // Pass
};

nc.call("test.avm1", responder, denseArray, sparseArray, mixedArray, fakeArray);
