// --- 1. SETUP THE DATA STRUCTURES ---

var arrLeadingZero = new Array();
arrLeadingZero["02"] = "val_02";

var arrLeadingSpace = new Array();
arrLeadingSpace[" 3"] = "val_space_3";

var arrTrailingSpace = new Array();
arrTrailingSpace["4 "] = "val_4_space";

var arrPlusSign = new Array();
arrPlusSign["+5"] = "val_plus_5";

var arrNegative = new Array();
arrNegative["-1"] = "val_neg_1";

var arrMixedEdge = new Array();
arrMixedEdge["\t6"] = "val_tab_6";
arrMixedEdge["007"] = "val_007";

trace("--- Original State (Pre-Serialization) ---");
trace("LeadingZero Length: " + arrLeadingZero.length);
trace("LeadingSpace Length: " + arrLeadingSpace.length);
trace("TrailingSpace Length: " + arrTrailingSpace.length);
trace("PlusSign Length: " + arrPlusSign.length);
trace("Negative Length: " + arrNegative.length);

// Helper function to trace keys
function traceKeys(name, arr) {
    for (var k in arr) {
        trace(name + " Key: '" + k + "' = " + arr[k]);
    }
}

// --- 2. TEST LOCALCONNECTION (Wire Serialization) ---
var lcReceiver = new LocalConnection();
lcReceiver.onReceiveArrays = function(aZ, aS, aT, aP, aN, aM) {
    trace("\n--- LC Deserialization Complete ---");
    trace("LC LeadingSpace Length: " + aS.length);

    traceKeys("LC LeadingZero", aZ);
    traceKeys("LC LeadingSpace", aS);
    traceKeys("LC TrailingSpace", aT);
    traceKeys("LC PlusSign", aP);
    traceKeys("LC Negative", aN);
    traceKeys("LC MixedEdge", aM);
};
lcReceiver.connect("amf0_edge_keys");

trace("\n--- Testing LocalConnection ---");
var lcSender = new LocalConnection();
lcSender.send("amf0_edge_keys", "onReceiveArrays", 
    arrLeadingZero, arrLeadingSpace, arrTrailingSpace, arrPlusSign, arrNegative, arrMixedEdge);

// --- 3. TEST SHAREDOBJECT (Disk Serialization) ---
trace("\n--- Testing SharedObject ---");
var so = SharedObject.getLocal("avm1_amf_edge_keys");
so.data.aZ = arrLeadingZero;
so.data.aS = arrLeadingSpace;
so.data.aT = arrTrailingSpace;
so.data.aP = arrPlusSign;
so.data.aN = arrNegative;
so.data.aM = arrMixedEdge;
so.flush();

var soRead = SharedObject.getLocal("avm1_amf_edge_keys");
trace("\n--- SO Deserialization Complete ---");
trace("SO LeadingSpace Length: " + soRead.data.aS.length);
traceKeys("SO LeadingSpace", soRead.data.aS);
traceKeys("SO TrailingSpace", soRead.data.aT);


// --- 4. TEST NETCONNECTION (AMF0 Wire Serialization) ---
trace("\n--- Testing NetConnection ---");
var nc = new NetConnection();
nc.connect("http://localhost:8000/");

var responder = new Object();
responder.onResult = function(res) { 
    trace("\n--- NC Result ---");
    trace("NC Result Object Length: " + res.length); 
};

nc.call("test.avm1_edge_keys", responder, 
    arrLeadingZero, arrLeadingSpace, arrTrailingSpace, arrPlusSign, arrNegative, arrMixedEdge);
