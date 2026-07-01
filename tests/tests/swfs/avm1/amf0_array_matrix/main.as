trace("--- START COMPREHENSIVE AMF0 MATRIX SUITE ---");

var denseArray:Array = new Array();
denseArray[0] = "dense_0";
denseArray[1] = "dense_1";

var sparseArray:Array = new Array();
sparseArray[0] = "sparse_0";
sparseArray[3] = "sparse_3"; // Gaps at indices 1 and 2

var mixedArray:Array = new Array();
mixedArray["a_prop"] = "value_a";
mixedArray[0] = "mixed_0";
mixedArray[1] = "mixed_1";
mixedArray["z_prop"] = "value_z";

// --------------------------------------------------
// PIPELINE 1: SharedObject (Local Disk)
// --------------------------------------------------
trace("\nExecuting SharedObject Test...");
var so = SharedObject.getLocal("RuffleMatrixStore");
so.data.dense = denseArray;
so.data.sparse = sparseArray;
so.data.mixed = mixedArray;
so.flush(); // Serializes to internal mock disk storage

var soRead = SharedObject.getLocal("RuffleMatrixStore");
inspectObject("SharedObject Dense Read", soRead.data.dense);
inspectObject("SharedObject Sparse Read", soRead.data.sparse);
inspectObject("SharedObject Mixed Read", soRead.data.mixed);


// --------------------------------------------------
// PIPELINE 2: LocalConnection (Inter-process IPC)
// --------------------------------------------------
trace("\nExecuting LocalConnection Test...");

// Setup the receiver instance to listen for incoming bytes
var lcReceiver = new LocalConnection();
lcReceiver.connect("_ruffle_matrix_channel");
lcReceiver.onUnpackPayload = function(receivedDense:Object, receivedSparse:Object, receivedMixed:Object) {
    trace("\n--- LocalConnection Async Callback Received ---");
    inspectObject("LocalConnection Dense Unpacked", receivedDense);
    inspectObject("LocalConnection Sparse Unpacked", receivedSparse);
    inspectObject("LocalConnection Mixed Unpacked", receivedMixed);
    trace("--- END COMPREHENSIVE AMF0 MATRIX SUITE ---");
};

// Instantiate the sender and fire the payload across the boundary
var lcSender = new LocalConnection();
lcSender.send("_ruffle_matrix_channel", "onUnpackPayload", denseArray, sparseArray, mixedArray);
trace(">> Bytes successfully serialized and committed to IPC stream.");


// --------------------------------------------------
// PIPELINE 3: NetConnection (Network AMF Remoting)
// --------------------------------------------------
trace("\nExecuting NetConnection Test...");
var nc = new NetConnection();
nc.connect("http://localhost:8000/");
var responder = new Object();
responder.onResult = function(result) {
    trace("Received result");
};

// Send all three arrays out over the wire
nc.call("verifyNetworkArrays", responder, denseArray, sparseArray, mixedArray);
