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

trace("--- Testing NetConnection Array Serialization ---");
// Pass all three arrays to the server
nc.call("test.arrays", responder, realArray, fakeArray, mixedArray);
