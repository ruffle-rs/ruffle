// --- 1. DEFINE CLASS STRUCTURES ---

function RegisteredClass() {
    this.foo = "baz";
}
Object.registerClass("com.test.RegisteredClass", RegisteredClass);

// --- 2. INSTANTIATE TEST VARIATIONS ---

// 1. Hidden instance constructor property (dontEnum: 1)
var objHiddenConstructor = new RegisteredClass();
objHiddenConstructor.foo = "hidden_ctor";
ASSetPropFlags(objHiddenConstructor, "constructor", 1);

// 2. Fully locked instance constructor property (dontEnum | dontDelete | readOnly: 7)
var objLockedConstructor = new RegisteredClass();
objLockedConstructor.foo = "locked_ctor";
ASSetPropFlags(objLockedConstructor, "constructor", 7);

// 3. SWF Version-Gated constructor (e.g., hidden from SWF 6 / bit 9: 0x200 / 512)
var objVersionGatedConstructor = new RegisteredClass();
objVersionGatedConstructor.foo = "version_gated_ctor";
// 0x80 (128) = SWF6 gate, 0x500 (1280) = SWF7 gate
ASSetPropFlags(objVersionGatedConstructor, "constructor", 0x80 | 0x500);

// 4. SWF Version-Gated __constructor__ property
var objVersionGatedMagicConstructor = new RegisteredClass();
objVersionGatedMagicConstructor.foo = "version_gated_magic_ctor";
ASSetPropFlags(objVersionGatedMagicConstructor, "__constructor__", 0x80 | 0x500);

// 5. Explicitly un-hidden prototype constructor (Flags cleared to 0)
var objClearedConstructorFlags = new RegisteredClass();
objClearedConstructorFlags.foo = "cleared_flags_ctor";
ASSetPropFlags(RegisteredClass.prototype, "constructor", 0, ~0);
ASSetPropFlags(RegisteredClass.prototype, "__constructor__", 0, ~0);

// --- 3. TEST NETCONNECTION (AMF0 Wire Serialization) ---
trace("--- Testing NetConnection Typed Objects ---");
var nc = new NetConnection();
nc.connect("http://localhost:8000/");

var responder = new Object();
responder.onResult = function(res) {
    // Pass
};

nc.call("test.avm1", responder, 
    objHiddenConstructor,
    objLockedConstructor,
    objVersionGatedConstructor,
    objVersionGatedMagicConstructor,
    objClearedConstructorFlags
);
