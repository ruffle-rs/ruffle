// ============================================================================
// TEST SUITE: Recursive TypedObject Serialization & Edge Cases
// ============================================================================

// --- 1. DEFINE CLASS STRUCTURES ---

function UnregisteredClass() {
    this.foo = "bar";
}

function RegisteredClass() {
    this.foo = "baz";
}
Object.registerClass("com.test.RegisteredClass", RegisteredClass);

function MismatchedClass() {
    this.foo = "qux";
}
Object.registerClass("CustomAMFAliasString", MismatchedClass);

function DoubleRegisteredClass() {
    this.foo = "double";
}
Object.registerClass("FirstAlias.Class", DoubleRegisteredClass);
Object.registerClass("SecondAlias.Class", DoubleRegisteredClass);

// --- 2. INSTANTIATE TEST VARIATIONS ---

// 1. Control: Plain anonymous object
var objPlain = new Object();
objPlain.foo = "plain";

// 2. Unregistered instance (Expects: 0x03)
var objUnregistered = new UnregisteredClass();

// 3. Registered instance (Expects: 0x10, "com.test.RegisteredClass")
var objRegistered = new RegisteredClass();

// 4. Mismatched Class instance
var objMismatched = new MismatchedClass();

// 5. Forged .constructor property (Tests mutation vs prototype)
var objForgedConstructor = new Object();
objForgedConstructor.foo = "forged_ctor";
objForgedConstructor.constructor = RegisteredClass;

// 6. Forged __proto__ link (Tests prototype chain resolution)
var objForgedProto = new Object();
objForgedProto.foo = "forged_proto";
objForgedProto.__proto__ = RegisteredClass.prototype;

// 7. Double registered instance (Tests alias selection)
var objDoubleRegistered = new DoubleRegisteredClass();

// 8. Nested Typed Object (Tests recursive type resolution)
var nested = new RegisteredClass();
nested.child = new RegisteredClass();

// 9. Mixed Hierarchy (Tests context switching between Typed 0x10 and Plain 0x03)
var mixed = new RegisteredClass();
mixed.nested_plain = objForgedConstructor;

// 10. Cyclic Object (The Ultimate Test: Tests reference cache/stack overflow)
var cyclic = new RegisteredClass();
cyclic.self = cyclic;

// --- 3. TEST NETCONNECTION ---

trace("--- Starting AMF0 Recursive Serialization Test ---");
var nc = new NetConnection();
nc.connect("http://localhost:8000/");

var responder = new Object();
responder.onResult = function(res) {
    trace("Serialization test complete.");
};

nc.call("test.avm1", responder, 
    objPlain, 
    objUnregistered, 
    objRegistered, 
    objMismatched, 
    objForgedConstructor, 
    objForgedProto, 
    objDoubleRegistered,
    nested,
    mixed,
    cyclic
);
