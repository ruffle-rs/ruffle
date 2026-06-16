// --- 1. DEFINE CLASS STRUCTURES ---

// Case A: A normal class with no registration
function UnregisteredClass() {
    this.foo = "bar";
}

// Case B: A standard registered class
function RegisteredClass() {
    this.foo = "baz";
}
Object.registerClass("com.test.RegisteredClass", RegisteredClass);

// Case C: A class registered with a completely mismatching alias string
function MismatchedClass() {
    this.foo = "qux";
}
Object.registerClass("CustomAMFAliasString", MismatchedClass);

// Case D: A class registered under two different names
function DoubleRegisteredClass() {
    this.foo = "double";
}
Object.registerClass("FirstAlias.Class", DoubleRegisteredClass);
Object.registerClass("SecondAlias.Class", DoubleRegisteredClass);

// --- 2. INSTANTIATE TEST VARIATIONS ---

// 1. Plain anonymous object (Control)
var objPlain = new Object();
objPlain.foo = "plain";

// 2. Unregistered instance (Should fall back to anonymous Object 0x03)
var objUnregistered = new UnregisteredClass();

// 3. Registered instance (Should encode as TypedObject 0x10 with "com.test.RegisteredClass")
var objRegistered = new RegisteredClass();

// 4. Mismatched Class instance (Verifies if Flash uses the alias string or function name)
var objMismatched = new MismatchedClass();

// 5. Forged .constructor property on a plain object
// (Checks if the encoder reads the mutable public .constructor property)
var objForgedConstructor = new Object();
objForgedConstructor.foo = "forged_ctor";
objForgedConstructor.constructor = RegisteredClass;

// 6. Forged __proto__ link on a plain object
// (Checks if the encoder resolves registration via the prototype chain)
var objForgedProto = new Object();
objForgedProto.foo = "forged_proto";
objForgedProto.__proto__ = RegisteredClass.prototype;

// 7. Double registered instance
// (Verifies which alias Flash uses during serialization—usually the last one registered)
var objDoubleRegistered = new DoubleRegisteredClass();


// --- 3. TEST NETCONNECTION (AMF0 Wire Serialization) ---
trace("--- Testing NetConnection Typed Objects ---");
var nc = new NetConnection();
nc.connect("http://localhost:8000/");

var responder = new Object();
responder.onResult = function(res) {
    // Pass
};

nc.call("test.avm1", responder, 
    objPlain, 
    objUnregistered, 
    objRegistered, 
    objMismatched, 
    objForgedConstructor, 
    objForgedProto,
    objDoubleRegistered
);
