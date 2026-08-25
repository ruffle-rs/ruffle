// Compile with:
//  mtasc -main Test.as -swf assets.swf -version 8 -out test.swf
class Test {

  // Stash some things before we monkey-patch the world.
  static var OBJECT = Object;
  static var ARRAY = Array;
  static var FUNCTION = Function;

  static var AN_ARRAY = [];
  static var A_STRING = new String("hello world");

  static var A_BITMAP = new flash.display.BitmapData(10, 10);
  static var A_POINT = new flash.geom.Point(5, -5);

  static var A_BLUR_FILTER = new flash.filters.BevelFilter(5, 45);
  static var A_BEVEL_FILTER = new flash.filters.BlurFilter(10, 10, 2);
  static var A_COLOR_MATRIX_FILTER = new flash.filters.ColorMatrixFilter([42, 9]);
  static var A_CONVOLUTION_FILTER = new flash.filters.ConvolutionFilter(2, 2, [1, 1, 1, 1]);
  static var A_DISPLACEMENT_MAP_FILTER = new flash.filters.DisplacementMapFilter(A_BITMAP, A_POINT, 1, 1, 10, 10);
  static var A_DROP_SHADOW_FILTER = new flash.filters.DropShadowFilter(5, 45);
  static var A_GLOW_FILTER = new flash.filters.GlowFilter(0xFF00FF);
  static var A_GRADIENT_BEVEL_FILTER = new flash.filters.GradientBevelFilter(5, 45);
  static var A_GRADIENT_GLOW_FILTER = new flash.filters.GradientGlowFilter(5, 45);
  static var BITMAP_FILTERS = [
    A_BLUR_FILTER, A_BEVEL_FILTER, A_COLOR_MATRIX_FILTER, A_CONVOLUTION_FILTER, A_DISPLACEMENT_MAP_FILTER,
    A_DROP_SHADOW_FILTER, A_GLOW_FILTER, A_GRADIENT_BEVEL_FILTER, A_GRADIENT_GLOW_FILTER
  ];


  static function main(current) {
    var v;

    trace("### Testing weird prototypes...");
    v = _global.Object;

    trace("// Object = {}");
    _global.Object = {};
    traceName({}, "{}");

    trace("// Object = { prototype: 5 }");
    _global.Object.prototype = 5;
    traceName({}, "{}");

    trace("// Object = { prototype: undefined }");
    _global.Object.prototype = undefined;
    traceName({}, "{}");

    trace("// Object = { prototype: <real Object>.prototype }");
    _global.Object.prototype = v.prototype;
    traceName({}, "{}");

    trace("// Object = 5");
    _global.Object = 5;
    traceName({}, "{}");

    trace("// delete Object");
    delete _global.Object;
    trace('"Object" in _global: ' + v.prototype.hasOwnProperty.call(_global, "Object"));
    traceName({}, "{}");

    _global.Object = v; // restore things back


    trace("");
    trace("### Monkey-patching-globals...");
    monkeyPatchClass(_global, "Object");
    monkeyPatchClass(_global, "Array");
    monkeyPatchClass(_global, "MovieClip");
    monkeyPatchClass(_global, "Button");
    monkeyPatchClass(_global, "TextField");
    monkeyPatchClass(_global, "Video");

    monkeyPatchClass(_global.flash.filters, "BevelFilter");
    monkeyPatchClass(_global.flash.filters, "BlurFilter");
    monkeyPatchClass(_global.flash.filters, "ColorMatrixFilter");
    monkeyPatchClass(_global.flash.filters, "ConvolutionFilter");
    monkeyPatchClass(_global.flash.filters, "DisplacementMapFilter");
    monkeyPatchClass(_global.flash.filters, "DropShadowFilter");
    monkeyPatchClass(_global.flash.filters, "GlowFilter");
    monkeyPatchClass(_global.flash.filters, "GradientBevelFilter");
    monkeyPatchClass(_global.flash.filters, "GradientGlowFilter");
    monkeyPatch(_global.flash, "filters", _global.flash.filters);

    trace("// hiding flash.filters..."); 
    _global.ASSetPropFlags(_global.flash, "filters", 0x2000 /* version 9 */);
    trace("flash.filters: " + _global.flash.filters);

    monkeyPatch(_global, "flash", _global.flash);

    // Do this last, as `monkeyPatchClass` uses it internally.
    monkeyPatchClass(_global, "Function");

    trace("");
    trace("### Testing Object");

    trace("// {}");
    traceName({});

    trace("// new OBJECT()");
    traceName(new OBJECT());

    trace("// OBJECT()");
    traceName(OBJECT());

    // TODO: test other Object-producing native APIs?
    // Hopefully they all behave identically.


    trace("");
    trace("### Testing Function");

    trace("// function() {}");
    traceName(function() {});

    trace("// FUNCTION()");
    traceName(FUNCTION());

    trace("// ASnative(...)");
    traceName(_global.ASnative(101, 0));


    trace("");
    trace("### Testing Array");

    trace("// []");
    traceName([]);

    trace("// new ARRAY()");
    traceName(new ARRAY());

    trace("// ARRAY()");
    traceName(ARRAY());

    trace("// A_STRING.split()");
    traceName(A_STRING.split());

    trace("// AN_ARRAY.slice()");
    traceName(AN_ARRAY.slice());
    // Skipped: other array-returning methods on Array

    // `MovieClip.filters` tested in the BitmapFilters section.

    trace("// A_COLOR_MATRIX_FILTER.matrix");
    traceName(A_COLOR_MATRIX_FILTER.matrix);

    trace("// A_CONVOLUTION_FILTER.matrix");
    traceName(A_CONVOLUTION_FILTER.matrix);

    // TODO: test other Array-producing native APIs?
    // Hopefully they all behave identically.


    trace("");
    trace("### Testing display objects")

    traceName(current, "current");

    trace("// current.createTextField(...)");
    traceName(current.createTextField("tf", 0, 0, 0, 100, 50));

    trace("// current.createEmptyMovieClip(...)");
    v = current.createEmptyMovieClip("mc1", 1);
    traceName(v);

    trace("// v.duplicateMovieClip(...)");
    v = v.duplicateMovieClip("mc2", 2);
    traceName(v);

    trace("// attachMovie('clip', ...)");
    var clip = current.attachMovie("clip", "clip", 3);
    traceName(clip, "clip");

    // DisplayObjects created from the timeline
    traceName(clip.nested, "clip.nested");
    traceName(clip.button, "clip.button");
    traceName(clip.textfield, "clip.textfield");
    traceName(clip.video, "clip.video");


    trace("");
    trace("### Testing BitmapFilters");

    trace("// current.filters");
    current.filters = BITMAP_FILTERS;
    v = current.filters;
    traceName(v);
    for (var i = 0; i < v.length; i++) {
      traceName(BITMAP_FILTERS[i], "BITMAP_FILTERS[" + i + "]");
      traceName(BITMAP_FILTERS[i].clone(), "BITMAP_FILTERS[" + i + "].clone()");
      traceName(v[i], "current.filters[" + i + "]");
    }
    current.filters = [];


    // TODO: test other classes that can be instantiated by AVM1 builtins;
    // see the list in ruffle's `SystemPrototypes`.

    fscommand("quit");
  }

  static function monkeyPatchClass(obj, name) {
    var old = obj[name];
    if (!(old instanceof Function)) {
      trace("ERROR: expected class " + name + " to be a function");
      return;
    }

    // Make new class
    var cls = function() {
      var a = arguments;
      trace("new " + this.className + "(" + a.join(", ") + ") called!");

      var zuper = super; // work around MTASC's checker.
      // super.apply(...) doesn't work :c
      switch (a.length) {
        case 0: return zuper();
        case 1: return zuper(a[0]);
        case 2: return zuper(a[0], a[1]);
        case 3: return zuper(a[0], a[1], a[2]);
        // Add more if needed.
        default:
          trace("ERROR: too many arguments for constructor: " + a.length);
      }
    };
    cls.prototype.__proto__ = old.prototype; // manual inheritance

    // Set class names
    old.prototype.className = name;
    cls.prototype.className = "My" + name;

    // Set a getter for cls.prototype, similarly to monkeyPatch.
    var getterPrototype = function () {
      trace("My" + name + ".prototype getter called");
    };
    OBJECT.prototype.addProperty.call(cls, "prototype", getterPrototype, null);

    monkeyPatch(obj, name, cls);
  }

  static function monkeyPatch(obj, name, cls) {
    // Set both as a property and a getter, for distinguishing what supports
    // getters and what doesn't.
    obj[name] = cls;
    var getter = function() {
      trace(name + " getter called!");
      return cls;
    };
    OBJECT.prototype.addProperty.call(obj, name, getter, null);
  }

  static function traceName(v, name) {
    if (name == null) name = "v";
    var t = typeof v;
    if (t !== "object" && t !== "movieclip" && t !== "function") {
      trace(name + " isn't an object: " + v + " (type: " + t + ")");
    } else if (v.className !== undefined) {
      trace(name + ".className: " + v.className);
    } else if (OBJECT.prototype.hasOwnProperty.call(v, "__proto__")) {
      trace("typeof " + name + ".__proto__: " + typeof v.__proto__);
    } else {
      trace(name + ": object without __proto__");
    }
  }
}
