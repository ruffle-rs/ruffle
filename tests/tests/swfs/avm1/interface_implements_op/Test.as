// Compile with:
//  mtasc -main Test.as -swf shim.swf -keep -out test.swf 
class Test {

  // shim.swf provides this function, which wraps the `ImplementsOp` AVM1 action.
  // arguments: (constructor, interfaces...)
  static var opcode__implements = _root.opcode__implements;

  static function main(current) {

    var MyClass = function() {};
    var obj = new MyClass();

    var InterfaceA = function() {};

    // Implementing no interfaces is a noop.
    trace("// MyClass implements <nothing>");
    opcode__implements(MyClass);
    trace("obj instanceof InterfaceA: " + (obj instanceof InterfaceA));
    
    trace("// MyClass implements InterfaceA");
    opcode__implements(MyClass, InterfaceA);
    trace("obj instanceof InterfaceA: " + (obj instanceof InterfaceA));

    // TODO: test various non-constructor arguments

    // Trying to implement interfaces a second time is completely ignored.
    trace("// MyClass implements InterfaceB");
    var InterfaceB = function() {};
    opcode__implements(MyClass,InterfaceB);
    trace("obj instanceof InterfaceA: " + (obj instanceof InterfaceA));
    trace("obj instanceof InterfaceB: " + (obj instanceof InterfaceB));
  }
}
