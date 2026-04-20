// Compile with:
//  mtasc -main -header 200:150:30 Test.as -swf test.swf -version 8
class Test {


  static function main(current) {
    Number.prototype.className = "Number";
    Boolean.prototype.className = "Boolean";
    String.prototype.className = "String";
    flash.display.BitmapData.prototype.className = "BitmapData";
    _global.className = "<_global>";

    MyNumber.prototype.className = "MyNumber";
    MyBoolean.prototype.className = "MyBoolean";
    MyString.prototype.className = "MyString";

    trace("Monkey-patching-globals...");
    // __proto__ should work, so let's test it.
    var proto = _global.__proto__ = { __proto__: _global.__proto__ };
    delete _global.Boolean;
    delete _global.Number;
    delete _global.String;

    addGetter(proto, "Boolean", MyBoolean);
    addGetter(proto, "Number", MyNumber);
    addGetter(proto, "String", MyString);

    trace("Testing primitive coercions...");
    trace("");
    
    testCoerced(true);
    testCoerced(42);
    testCoerced("hello");

    trace('// "world".length');
    trace("result: " + "world".length);

    var v = "callme";
    trace('// "callme"()');
    trace("result: " + v());
    trace('// new "callme"()');
    trace("result: " + new v());

    trace("");
    trace("Testing loookup logic...");
    trace("");

    testCoerced(undefined);
    testCoerced(null);

    trace("// delete Number; __resolve = () => MyNumber");
    delete _global.Number;
    delete proto.Number;
    _global.__resolve = function(name) {
      trace("__resolve(" + name + ") called!");
      return MyNumber;
    };

    testCoerced(42);

    trace("// Number = MyNumber");
    _global.Number = MyNumber;
    testCoerced(42);

    trace("// Number.prototype = true");
    _global.Number.prototype = true;
    testCoerced(42);

    trace('// Number = "some string"');
    _global.Number = "some string";
    testCoerced(42);

    trace('// Number = BitmapData');
    _global.Number = flash.display.BitmapData;
    testCoerced(42);

    trace('// Number = {}');
    _global.Number = {};
    testCoerced(42);

    trace('// Number = Boolean');
    _global.Number = Boolean;
    testCoerced(42);

    trace("Done!");
    fscommand("quit");
  }


  static function addGetter(obj, name, val) {
    var getter = function() {
      trace(name + " getter called!");
      return val;
    };
    Object.addProperty.call(obj, name, getter, null);
  }

  static var TEST_COERCED_INNER = function(val) {
    trace("  coerced: " + this);
    if (this === _global) {
      trace("  is _global!");
    } else {
      trace("  typeof: " + typeof this);
      trace("  typeof __proto__: " + typeof this.__proto__);
      trace("  className: " + this.className);
    }
  };

  static function testCoerced(val) {
    trace("// " + val + " coerced to 'this'");
    TEST_COERCED_INNER.call(val, val);
    trace("// " + val + ".className");
    trace("  className: " + val.className);
    trace("");
  }

  static function monkeyPatch(obj, name, cls) {
    // Set both as a property and a getter, for distinguishing what supports
    // getters and what doesn't.
    obj[name] = cls;
    var getter = function() {
      trace(name + " getter called!");
      return cls;
    };
    Object.prototype.addProperty.call(obj, name, getter, null);
  }

}
