// Compile with:
//  mtasc -main -header 200:150:30 Test.as -swf test.swf -version 6
class Test {

  static var NAMES = [
    // don't test Math.random(); it is... random
    // unary
    "abs", "sin", "cos", "tan", "exp", "log", "sqrt", "round", "floor", "ceil", "atan", "asin", "acos",
    // binary
    "atan2", "min", "max", "pow"
  ];

  static function main(current) {
    var nan = 0/0, inf = 1/0;

    for (var i = 0; i < NAMES.length; i++) {
      var name = NAMES[i];

      testNilary(name);
      testUnary(name, null);
      testUnary(name, undefined);
      testUnary(name, true);
      testUnary(name, false);
      testUnary(name, -1);
      testUnary(name, nan);
      testUnary(name, inf);
      testUnary(name, -inf);

      switch(name) {
        case "round":
          testUnary(name, 23.2);
          testUnary(name, 23.7);
          testUnary(name, -23.2);
          testUnary(name, -23.7);
          // fallthrough
        case "ceil":
        case "floor":
          testUnary(name, 12.5);
          testUnary(name, -12.5);
          break;

        case "sin":
          testUnary(name, Math.PI/2);
          break;

        case "cos":
          testUnary(name, Math.PI);
          break;

        case "pow":
        case "min":
        case "max":
        case "atan2":
          testBinary(name, 2, null);
          testBinary(name, 2, undefined);
          testBinary(name, null, 2);
          testBinary(name, undefined, 2);
          break;
      }

      testBinary(name, noisy(2), noisy(1));
      testTrinary(name, noisy(3), noisy(2), noisy(1));
      trace("");
    }

    // Test random manually, ignoring the result.
    trace("// Math.random({ v: 3 }, { v: 2 }, { v: 1 })");
    Math.random(noisy(3), noisy(2), noisy(1));
    trace("#RANDOM#");
    trace("");

    // Test invalid ASnative index.
    var invalid = _global.ASnative(200, 50);
    trace("// ASnative(200, 50)()");
    trace(invalid());
    trace("// ASnative(200, 50)({ v: 3 }, { v: 2 }, { v: 1 })");
    trace(invalid(noisy(3), noisy(2), noisy(1)));
    trace("");

    // Test behavior on throwing coercions.
    var throwA = {
      valueOf: function() {
        trace("will throw A!");
        throw "A";
      }
    };
    var throwB = {
      valueOf: function() {
        trace("will throw B!");
        throw "B";
      }
    };
    try {
      trace("// Math.min({ throw A }, { throw B })");
      var result = Math.min(throwA, throwB);
      trace("result: " + result);
    } catch (e) {
      trace("caught: " + e);
    }
    
    fscommand("quit");
  }

  static var NOISY_VALUE_OF = function() {
    trace("valueOf called: " + this.v);
    return this.v;
  };

  static function noisy(v) {
    return { v: v, valueOf: NOISY_VALUE_OF };
  }

  static function testNilary(name) {
    var f = _global.Math[name];
    trace("// Math." + name + "()");
    trace(f());
  }

  static function testUnary(name, arg) {
    var f = _global.Math[name];
    trace("// Math." + name + "(" + argToString(arg) + ")");
    trace(f(arg));
  }

  static function testBinary(name, arg1, arg2) {
    var f = _global.Math[name];
    trace("// Math." + name + "(" + argToString(arg1) + ", " + argToString(arg2) + ")");
    trace(f(arg1, arg2));
  }

  static function testTrinary(name, arg1, arg2, arg3) {
    var f = _global.Math[name];
    trace("// Math." + name + "(" + argToString(arg1) + ", " + argToString(arg2) + ", " + argToString(arg3) + ")");
    trace(f(arg1, arg2, arg3));
  }

  static function argToString(arg) {
    if (arg === null) return "null";
    if (arg === undefined) return "undefined";
    switch (typeof arg) {
      case "object": return "{ v: " + arg.v + " }";
      case "string": return "'" + arg + "'";
      default: return "" + arg;
    }
  }
}
