// Compile with:
//  mtasc -main Test.as -swf shim.swf -keep -out test.swf 
class Test {

  // shim.swf provides this function, which wraps the `ImplementsOp` AVM1 action.
  // arguments: (constructor, interfaces...)
  static var opcode__implements = _root.opcode__implements;

  static function main(current) {
    var swf6 = current.createEmptyMovieClip("swf6", 10000);
    var loader = new MovieClipLoader();
    loader.loadClip("child6.swf", swf6);
    loader.addListener({
      onLoadInit: function() {
        Test.test(swf6);
        fscommand("quit");
      }
    });
  }

  static function test(swf6) {
    _global.String = NoisyString;

    var MyClass = function() {};
    var obj = new MyClass();

    var InterfaceA = function() {};
    var FakeInterfaceA = { prototype: InterfaceA.prototype };

    // This checks that properties are ignored.
    addGetterOnField(InterfaceA, "InterfaceA", "prototype");
    trace("InterfaceA.prototype is: " + InterfaceA.prototype);
    trace("");

    // Implementing no interfaces is a noop.
    trace("// MyClass implements <nothing>");
    opcode__implements(MyClass);
    trace("obj instanceof InterfaceA: " + (obj instanceof InterfaceA));
    
    trace("// MyClass implements InterfaceA");
    opcode__implements(MyClass, InterfaceA);
    trace("obj instanceof InterfaceA: " + (obj instanceof InterfaceA));
    trace("obj instanceof fake InterfaceA: " + (obj instanceof FakeInterfaceA));
    trace("obj instanceof { __proto__: InterfaceA }: " + (obj instanceof { __proto__: InterfaceA }));
    trace("in swf6 { obj instanceof InterfaceA }: " + (swf6.isInstanceOf(obj, InterfaceA)));

    // Trying to implement interfaces a second time is completely ignored.
    trace("// MyClass implements InterfaceB");
    var InterfaceB = function() {};
    opcode__implements(MyClass, InterfaceB);
    trace("obj instanceof InterfaceA: " + (obj instanceof InterfaceA));
    trace("obj instanceof InterfaceB: " + (obj instanceof InterfaceB));

    // Nested interfaces should work.
    trace("// InterfaceA implements NestedInterfaceA");
    var NestedInterfaceA = function() {};
    opcode__implements(InterfaceA, NestedInterfaceA);
    trace("obj instanceof NestedInterfaceA: " + (obj instanceof NestedInterfaceA));

    // But not superclasses of interfaces.
    trace("// NestedInterfaceA extends X");
    var X = function() {};
    NestedInterfaceA.prototype.__proto__ = X.prototype;
    trace("obj instance of X: " + (obj instanceof X));

    trace("");
    
    var MyClass2 = function() {};
    obj = new MyClass2();

    // Testing objects with interesting prototypes.
    var InterfaceInt = { prototype: 42 };
    addGetterOnField(InterfaceInt, "InterfaceInt", "prototype");
    var InterfaceString = { prototype: "hello" };
    var InterfaceRoot = { prototype: _root };
    var McInterfaceB = _root.createEmptyMovieClip("mcInterfaceB", 1);
    McInterfaceB.prototype = InterfaceB.prototype;

    trace("// MyClass2 implements InterfaceInt, InterfaceString, InterfaceRoot, FakeInterfaceA, McInterfaceB");
    opcode__implements(MyClass2, InterfaceInt, InterfaceString, InterfaceRoot, FakeInterfaceA, McInterfaceB);
    trace("obj instanceof InterfaceInt: " + (obj instanceof InterfaceInt));
    trace("obj instanceof InterfaceString: " + (obj instanceof InterfaceString));
    trace("obj instanceof InterfaceRoot: " + (obj instanceof InterfaceRoot));
    trace("obj instanceof InterfaceA: " + (obj instanceof InterfaceA));
    trace("obj instanceof FakeInterfaceA: " + (obj instanceof FakeInterfaceA));
    trace("obj instanceof InterfaceB: " + (obj instanceof InterfaceB));
    trace("obj instanceof McInterfaceB: " + (obj instanceof McInterfaceB));

    trace("");

    // Testing non-object arguments.
    var MyClass3 = function() {};
    obj = new MyClass3();

    trace('// null implements MyClass');
    opcode__implements(null, MyClass);
    trace('// "left" implements MyClass3');
    opcode__implements("left", MyClass3);
    trace('// "left" implements "right"');
    opcode__implements("left", "right");
    trace('// MyClass3 implements "right", InterfaceA');
    opcode__implements(MyClass3, "right", InterfaceA);
    trace("obj instanceof NoisyString: " + (obj instanceof NoisyString));
    trace("obj instanceof InterfaceA: " + (obj instanceof InterfaceA));

    trace("");

    // Implementing interfaces with all-invalid arguments still makes further usages noops.
    var MyClass4 = function() {};
    obj = new MyClass4();
  
    trace('// MyClass4 implements undefined');
    opcode__implements(MyClass4, undefined);
    trace("obj instanceof undefined: " + (obj instanceof undefined));

    trace('// MyClass4 implements InterfaceA');
    opcode__implements(MyClass4, InterfaceA);
    trace("obj instanceof InterfaceA: " + (obj instanceof InterfaceA));
    
    trace("");

    // The interface can be implemented at the last minute...
    var MyClass5 = function() {};
    obj = new MyClass5();
    var LazyInterfaceA = { prototype: "trigger" };

    NoisyString.ON_NEW = function(self) {
      trace("Lazily: MyClass5 implements LazyInterfaceA!");
      LazyInterfaceA.prototype = self;
      Test.opcode__implements(MyClass5, LazyInterfaceA);
    };

    trace("obj instanceof LazyInterfaceA: " + (obj instanceof LazyInterfaceA));

    NoisyString.ON_NEW = null;
    trace("");

    // ...but modifying the prototype of an already-implemented interface does nothing.
    var MyClass6 = function() {};
    obj = new MyClass6();

    var LazyInterfaceB = {};
    trace("// MyClass6 implements LazyInterfaceB");
    opcode__implements(MyClass6, LazyInterfaceB);
    trace("// LazyInterfaceB.prototype = {}");
    LazyInterfaceB.prototype = {};
    trace("obj instanceof LazyInterfaceB: " + (obj instanceof LazyInterfaceB));

    fscommand("quit");
  }

  static function addGetterOnField(obj, objName, name) {
    var value = obj[name];
    var getter = function() {
      trace(objName + "." + name + " getter called!");
      return value;
    };
    Object.prototype.addProperty.call(obj, name, getter, null);
  }
}
