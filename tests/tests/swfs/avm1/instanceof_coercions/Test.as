// Compile with:
//  mtasc -main -version 8 Test.as -swf shim.swf -keep -out test.swf
class Test {

  // shim.swf provides this function, which wraps the `CastOp` AVM1 action.
  // arguments: (object, constructor)
  // It is normally available through `catch(e: Class)`, but direct access
  // makes for simpler test code, and allows inspecting the actual return value.
  static var opcode__cast = _root.opcode__cast;

  static function main(current) {
    _global.String = NoisyString;

    trace('// obj: "left", class: "right"');
    check("left", "right");
    trace('// obj: "left", class: Object');
    check("left", Object);
    trace('// obj: {}, class: "right"');
    check({}, "right");
    trace('');

    trace('// obj: {}, class: { prototype: Object.prototype }');
    check({}, { prototype: Object.prototype });

    trace('// obj: { __proto__: "left" }, class: { prototype: "right" }');
    check({ __proto__: "left" }, { prototype: "right" });
    
    trace('// obj: { __proto__: "proto" }, class: { prototype: "proto" }');
    check({ __proto__: "proto" }, { prototype: "proto" });

    trace('// obj: { __proto__: null }, class: { prototype: null }');
    check({ __proto__: null }, { prototype: null });

    // Prototype properties don't work...
    var fakeClass = {};
    fakeClass.addProperty("prototype", function() {
      trace("prototype property called!");
      return "property proto";
    }, null);
    trace('// obj: {}, class: { get prototype(): ... }');
    trace('.prototype is: ' + fakeClass.prototype);
    check({}, fakeClass);

    // ...and neither do 'indirect' prototypes...
    fakeClass = { __proto__: { prototype: {}}};
    trace('// obj: { __proto__: X }, class: { __proto__: { prototype: X }}');
    check({ __proto__: fakeClass.prototype }, fakeClass);

    // ...nor prototypes behind 'super'.
    var FakeClassSuper = function() { this.zuper = super; };
    FakeClassSuper.prototype = { __proto__: { prototype: "super proto" }};
    fakeClass = (new FakeClassSuper()).zuper;
    trace('// obj: {}, class: super { prototype: ... }');
    trace('.prototype is: ' + fakeClass.prototype);
    check({}, fakeClass);

    // SWF version flags are ignored.
    fakeClass = { prototype: "SWFv9 proto" };
    _global.ASSetPropFlags(fakeClass, "prototype", 0x2000 /* version 9 */);
    trace('// obj: {}, class: { prototype(SWFv9): ... }');
    trace('.prototype is: ' + fakeClass.prototype);
    check({}, fakeClass);

    trace('');

    var movieclip = current.createEmptyMovieClip("mc", 1);
    movieclip.prototype = Object.prototype;
    testsForMovieClip(movieclip);
    trace('// kill movieclip');
    movieclip.removeMovieClip();
    testsForMovieClip(movieclip);

    fscommand('quit');
  }

  static function testsForMovieClip(movieclip) {
    trace('// obj: movieclip, class: "right"');
    check(movieclip, "right");
    trace('// obj: movieclip, class: Object');
    check(movieclip, Object);
    trace('// obj: movieclip, class: { prototype: "proto" }');
    check(movieclip, { prototype: "proto" });
    trace('// obj: {}, class: MovieClip { prototype: Object.prototype }');
    check({}, movieclip);
    trace('// obj: { __proto__: movieclip }, class: { prototype: movieclip }');
    check({ __proto__: movieclip }, { prototype: movieclip });
  }

  static function check(obj, klass) {
    trace("instanceof: " + (obj instanceof klass));
    trace("typeof cast: " + typeof opcode__cast(obj, klass));
  }
}
