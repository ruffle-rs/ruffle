// Compile with:
//  mtasc -main -header 200:150:30 Test.as -swf test.swf -version 8
class Test {

  static function main(current) {
    var obj;

    trace("#1: `super.foobar()` never does primitive-to-object coercions");

    String.prototype.foobar = function() {
      trace("String.prototype.foobar called!");
      return "string-foobar";
    };

    current.__proto__ = {
      foobar: function() {
        trace("_root.__proto__.foobar called!");
        return "_root-foobar";
      }
    };

    obj = {
      foobar: function() {
        trace("obj.foobar called!");
        trace("super.foobar(): " + super.foobar());
        return "obj-foobar";
      }
    };
  
    trace("// obj.__proto__ = _root");
    obj.__proto__ = current;
    trace("obj.foobar(): " + obj.foobar());

    trace("// obj.__proto__ = new String('hello')");
    obj.__proto__ = new String("hello");
    trace("obj.foobar(): " + obj.foobar());


    trace("// obj.__proto__ = 'hello'");
    obj.__proto__ = "hello";
    trace("obj.foobar(): " + obj.foobar());

    trace("");
    trace("#2: `super()` never calls `__resolve`");

    var __constructor__ = function() {
      trace("__constructor__ called!");
      return "constructed";
    };

    trace("// obj.__constructor__ = ...");
    obj = {
      __constructor__ : __constructor__,
      foobar: function() {
        trace("obj.foobar called!");
        var zuper = super; // to bypass MTASC checks
        trace("super(): " + zuper());
      }
    };
    obj.foobar();

    trace("// __proto__.__resolve = () => __constructor__");
    obj.__proto__ = {
      __resolve: function(name) {
        trace("__resolve called with: " + name);
        return __constructor__;
      }
    };
    obj.foobar();

    // one extra level of nesting than strictly required.
    trace("// __proto__.__proto__.__constructor__ = ...");
    obj.__proto__.__proto__ = {
      __constructor__: __constructor__
    };
    obj.foobar();


    obj.__proto__.addProperty(
      "__constructor__",
      function() {
        trace("__constructor__ property called!");
        return __constructor__;
      },
      null
    );
    trace("// __proto__.addProperty('__constructor__', ...)");
    obj.foobar();

    trace("// __proto__ = makeSuperWith(__proto__)");
    obj.__proto__ = makeSuperWith(obj.__proto__);
    obj.foobar();

    trace("// (__proto__ = _root).__constructor__ = ...");
    (obj.__proto__ = _root).__constructor__ = __constructor__;
    obj.foobar();


    fscommand("quit");
  }

  static function makeSuperWith(obj) {
    var helper = {
      __proto__: { __proto__: obj },
      getSuper: function() { return super; }
    };
    return helper.getSuper();
  }
}
