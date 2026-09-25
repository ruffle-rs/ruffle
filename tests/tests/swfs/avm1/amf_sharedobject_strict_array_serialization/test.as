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

// --- 2. TEST SHAREDOBJECT (AMF0 LSO Serialization) ---
trace("--- Testing SharedObject ---");

var so = SharedObject.getLocal("strict_array_test");

so.data.denseArray = denseArray;
so.data.sparseArray = sparseArray;
so.data.mixedArray = mixedArray;
so.data.fakeArray = fakeArray;
so.data.superFakeArray = superFakeArray;
so.data.demotedArray = demotedArray;
so.data.protoOnlyArray = protoOnlyArray;

so.flush();
trace("SharedObject flushed successfully.");
