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

// Case E: Same alias re-registered to a different constructor
function AliasOriginalClass() {
    this.foo = "alias_original";
}
function AliasReplacementClass() {
    this.foo = "alias_replacement";
}
Object.registerClass("SharedAlias.Class", AliasOriginalClass);
Object.registerClass("SharedAlias.Class", AliasReplacementClass);

// Case F: Constructor registered under three aliases
function TripleRegisteredClass() {
    this.foo = "triple";
}
Object.registerClass("TripleAlias.One", TripleRegisteredClass);
Object.registerClass("TripleAlias.Two", TripleRegisteredClass);
Object.registerClass("TripleAlias.Three", TripleRegisteredClass);

// Case G: Instance created before alias changes
function LateRegistrationClass() {
    this.foo = "late";
}
Object.registerClass("LateAlias.First", LateRegistrationClass);
// Must be created before the alias is changed to properly test.
var objLateRegistration = new LateRegistrationClass();
Object.registerClass("LateAlias.Second", LateRegistrationClass);

// Case H: Prototype getter shadowed by an instance field
function ShadowedGetterClass() {}

ShadowedGetterClass.prototype.addProperty(
    "foo",
    function() {
        trace("prototype getter: foo");
        return "getter";
    },
    null
);

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

// 8. Original constructor after its alias was reassigned
// (Should verify whether AliasOriginalClass is still treated as typed or
// falls back to an anonymous object after its alias is stolen.)
var objAliasOriginal = new AliasOriginalClass();

// 9. Replacement constructor
// (Should serialize using "SharedAlias.Class".)
var objAliasReplacement = new AliasReplacementClass();

// 10. Triple registered instance
// (Verifies which alias Flash chooses when more than two aliases exist.)
var objTripleRegistered = new TripleRegisteredClass();

// 11. Getter-backed "constructor" property
// (Checks whether serialization invokes a getter for "constructor".)
var objGetterConstructor = new Object();
objGetterConstructor.foo = "getter_ctor";
objGetterConstructor.addProperty(
    "constructor",
    function() {
        trace("getter: constructor");
        return RegisteredClass;
    },
    null
);

// 12. Getter-backed "__constructor__" property
// (Checks whether serialization invokes a getter for "__constructor__".)
var objGetterMagicConstructor = new Object();
objGetterMagicConstructor.foo = "getter_magic_ctor";
objGetterMagicConstructor.addProperty(
    "__constructor__",
    function() {
        trace("getter: __constructor__");
        return RegisteredClass;
    },
    null
);

// 13. Object with no prototype chain
var objNoProto = new Object();
objNoProto.foo = "noproto";
objNoProto.__proto__ = undefined;

// 14. Object with a primitive __proto__
var objPrimitiveProto = new Object();
objPrimitiveProto.foo = "primitiveproto";
objPrimitiveProto.__proto__ = 42;

// 15. Object with a throwing getter
var objThrowingGetter = new Object();
objThrowingGetter.foo = "throwing_getter";
objThrowingGetter.addProperty("badProp", function() {
    throw new Error("AMF Getter Error");
}, null);

// 16. Object with throwing __resolve
var objThrowingResolve = new Object();
objThrowingResolve.foo = "throwing_resolve";
objThrowingResolve.__resolve = function(name) {
    throw new Error("AMF Resolve Error for " + name);
};

// 17. Throwing 'constructor' property
var objThrowingConstructor = new Object();
objThrowingConstructor.foo = "throwing_ctor";
objThrowingConstructor.addProperty("constructor", function() {
    throw new Error("Constructor retrieval error");
}, null);

// 18. Throwing '__constructor__' property
var objThrowingMagicConstructor = new Object();
objThrowingMagicConstructor.foo = "throwing_magic_ctor";
objThrowingMagicConstructor.addProperty("__constructor__", function() {
    throw new Error("Magic Constructor retrieval error");
}, null);

// 19. Infinite recursion
var objHardErrorGetter = new Object();
objHardErrorGetter.foo = "hard_error";
objHardErrorGetter.addProperty("stackOverflowProp", function() {
    return this.stackOverflowProp;
}, null);

// 20. Prototype getter shadowed by an instance field
var objShadowedGetter = new ShadowedGetterClass();
objShadowedGetter.foo = "instance";

// 21. __resolve throws during constructor lookup
var objResolveConstructorThrow = new Object();
objResolveConstructorThrow.foo = "resolve_ctor_throw";
delete objResolveConstructorThrow.constructor;
delete objResolveConstructorThrow.__constructor__;
objResolveConstructorThrow.__resolve = function(name) {
    if (name == "constructor" || name == "__constructor__") {
        throw new Error("__resolve constructor error");
    }
    return undefined;
};

// 22. Constructor inherited from a deeper prototype
var protoLevel2 = {};
protoLevel2.constructor = RegisteredClass;

var protoLevel1 = {};
protoLevel1.__proto__ = protoLevel2;

var objDeepPrototype = new Object();
objDeepPrototype.foo = "deep_proto";
delete objDeepPrototype.constructor;
objDeepPrototype.__proto__ = protoLevel1;

// 23. True Shadowed Prototype Getter
var fakeProto = new Object();
fakeProto.addProperty("foo", function() { return "proto_getter"; }, null);

var objTrueShadow = new Object();
objTrueShadow.foo = "normal_instance_value"; // Create normal property FIRST
objTrueShadow.__proto__ = fakeProto;         // Apply virtual prototype SECOND

// 24. Prototype Getter and Setter
function GetterSetterClass() {
    this._foo = "internal";
}
GetterSetterClass.prototype.addProperty(
    "foo", 
    function() { return this._foo; }, 
    function(val) { this._foo = val; }
);
var objGetterSetter = new GetterSetterClass();
objGetterSetter.foo = "setter_called"; // Invokes setter, does not create a normal property

// 25. Hidden instance constructor property (dontEnum: 1)
var objHiddenConstructor = new RegisteredClass();
objHiddenConstructor.foo = "hidden_ctor";
ASSetPropFlags(objHiddenConstructor, "constructor", 1);

// 26. Fully locked instance constructor property (dontEnum | dontDelete | readOnly: 7)
var objLockedConstructor = new RegisteredClass();
objLockedConstructor.foo = "locked_ctor";
ASSetPropFlags(objLockedConstructor, "constructor", 7);

// 27. SWF Version-Gated constructor (e.g., hidden from SWF 6 / bit 9: 0x200 / 512)
var objVersionGatedConstructor = new RegisteredClass();
objVersionGatedConstructor.foo = "version_gated_ctor";
// 0x80 (128) = SWF6 gate, 0x500 (1280) = SWF7 gate
ASSetPropFlags(objVersionGatedConstructor, "constructor", 0x80 | 0x500);

// 28. SWF Version-Gated __constructor__ property
var objVersionGatedMagicConstructor = new RegisteredClass();
objVersionGatedMagicConstructor.foo = "version_gated_magic_ctor";
ASSetPropFlags(objVersionGatedMagicConstructor, "__constructor__", 0x80 | 0x500);

// 29. Explicitly un-hidden prototype constructor (Flags cleared to 0)
var objClearedConstructorFlags = new RegisteredClass();
objClearedConstructorFlags.foo = "cleared_flags_ctor";
ASSetPropFlags(RegisteredClass.prototype, "constructor", 0, ~0);
ASSetPropFlags(RegisteredClass.prototype, "__constructor__", 0, ~0);

// 30. Delete Object prototype constructor properties
delete Object.prototype.constructor;
delete Object.prototype.__constructor__;
var objOrphan = new Object();
objOrphan.foo = "orphan_chain";

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
    objDoubleRegistered,
    objAliasOriginal,
    objAliasReplacement,
    objTripleRegistered,
    objLateRegistration,
    objGetterConstructor,
    objGetterMagicConstructor,
    objNoProto,
    objPrimitiveProto,
    objThrowingGetter,
    objThrowingResolve,
    objThrowingConstructor,
    objThrowingMagicConstructor,
    objHardErrorGetter,
    objShadowedGetter,
    objResolveConstructorThrow,
    objDeepPrototype,
    objTrueShadow,
    objGetterSetter,
    objHiddenConstructor,
    objLockedConstructor,
    objVersionGatedConstructor,
    objVersionGatedMagicConstructor,
    objClearedConstructorFlags,
    objOrphan
);
