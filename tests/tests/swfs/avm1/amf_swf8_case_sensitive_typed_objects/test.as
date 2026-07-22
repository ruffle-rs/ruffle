// --- 1. DEFINE CLASS STRUCTURES ---

function BasicClass() {
    this.foo = "basic";
}
Object.registerClass("com.test.BasicClass", BasicClass);

function OriginalClass() {
    this.foo = "original";
}
function OverwriteClass() {
    this.foo = "overwrite";
}

Object.registerClass("sharedalias", OriginalClass);
Object.registerClass("SharedAlias", OverwriteClass);

// --- 2. INSTANTIATE TEST VARIATIONS ---

// 1. Basic typed object
var objBasic = new BasicClass();

// 2. The original constructor
var objOriginal = new OriginalClass();

// 3. The replacement constructor
var objOverwrite = new OverwriteClass();

// --- 3. TEST NETCONNECTION (AMF0 Wire Serialization) ---
trace("--- Testing NetConnection Typed Objects (SWF 8 Case Sensitive) ---");
var nc = new NetConnection();
nc.connect("http://localhost:8000/");

var responder = new Object();
responder.onResult = function(res) {
    // Pass
};

nc.call("test.avm1", responder, 
    objBasic, 
    objOriginal, 
    objOverwrite
);
