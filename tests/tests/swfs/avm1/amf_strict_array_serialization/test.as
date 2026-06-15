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

var throwingArray = new Array();
throwingArray[0] = "safe_0";
throwingArray.addProperty("1", function() { throw new Error("Throwing getter!"); }, null);
throwingArray.addProperty("length", function() { throw new Error("Throwing length!"); }, null);

var FAKE_ARRAY_PROTO = {
    __proto__: Array.prototype,
    __constructor__: Array,
    __initializeNative: function() {
        super();
    }
};

var superFakeArray = new Object();
superFakeArray.__proto__ = FAKE_ARRAY_PROTO;
superFakeArray.__initializeNative();
superFakeArray[0] = "super_fake_0";
superFakeArray.__proto__ = Array.prototype;

superFakeArray[5] = "litmus";
trace("Did super() upgrade instance? " + (superFakeArray.length === 6 ? "YES" : "NO"));
delete superFakeArray[5];
superFakeArray.length = 1;


var demotedArray = new Array("demoted_0", "demoted_1");
demotedArray.__proto__ = Object.prototype;
demotedArray.__constructor__ = Object;


var protoOnlyArray = new Object();
protoOnlyArray.__proto__ = Array.prototype;
protoOnlyArray[0] = "cosplay_0";
protoOnlyArray.length = 1;

// --- 2. TEST NETCONNECTION (AMF0 Wire Serialization) ---
trace("--- Testing NetConnection ---");
var nc = new NetConnection();
nc.connect("http://localhost:8000/");

var responder = new Object();
responder.onResult = function(res) {
  // Pass
};

nc.call("test.avm1", responder, denseArray, sparseArray, mixedArray, fakeArray, throwingArray, superFakeArray, demotedArray, protoOnlyArray);
