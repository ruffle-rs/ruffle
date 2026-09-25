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

// NEW: Native Types (Top Level)
var topDate = new Date(1672531200000); // Fixed UTC Timestamp (Jan 1, 2023)
var topXML = new XML("<root><node attr='test'>AVM1</node></root>");

// NEW: Nested Object (Testing recursion fixes for ALL types)
var nestedContainer = new Object();
nestedContainer.deepDense = new Array("deep_0", "deep_1");
nestedContainer.deepSparse = new Array();
nestedContainer.deepSparse[0] = "deep_s0";
nestedContainer.deepSparse[3] = "deep_s3";
nestedContainer.deepDate = new Date(1672531200000);
nestedContainer.deepXML = new XML("<nested>data</nested>");


// --- 2. TEST LOCALCONNECTION (Wire Serialization) ---
var lcReceiver = new LocalConnection();
lcReceiver.onReceiveArrays = function(d, s, m, f, date, xml, n) {
    trace("LC Deserialization Complete: " + d[0] + ", " + s[5] + ", " + date.getTime() + ", " + xml.firstChild.nodeName + ", " + n.deepDate.getTime());
};
lcReceiver.connect("amf0_test_connection");

trace("--- Testing LocalConnection ---");
var lcSender = new LocalConnection();
lcSender.send("amf0_test_connection", "onReceiveArrays", denseArray, sparseArray, mixedArray, fakeArray, topDate, topXML, nestedContainer);


// --- 3. TEST SHAREDOBJECT (Disk Serialization) ---
trace("--- Testing SharedObject ---");
var so = SharedObject.getLocal("avm1_amf_test");
so.data.d = denseArray;
so.data.s = sparseArray;
so.data.m = mixedArray;
so.data.f = fakeArray;
so.data.date = topDate;
so.data.xml = topXML;
so.data.n = nestedContainer;
so.flush();

var soRead = SharedObject.getLocal("avm1_amf_test");
trace("SO Deserialization Complete: " + soRead.data.date.getTime() + ", " + soRead.data.n.deepXML.firstChild.nodeName);


// --- 4. TEST NETCONNECTION (AMF0 Wire Serialization) ---
trace("--- Testing NetConnection ---");
var nc = new NetConnection();
nc.connect("http://localhost:8000/");

var responder = new Object();
responder.onResult = function(res) { trace("NC Result"); };

nc.call("test.avm1", responder, denseArray, sparseArray, mixedArray, fakeArray, topDate, topXML, nestedContainer);
