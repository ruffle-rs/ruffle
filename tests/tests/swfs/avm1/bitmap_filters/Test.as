// Compile with:
//  mtasc -main -version 8 Test.as -swf assets.swf -out test.swf 
class Test {

  static function main(current) {
    {
      var f = constructAndTestClone("BitmapFilter");
    }

    trace("");

    {
      var f = constructAndTestClone("BevelFilter");
      testMovieClipFilterSetter(current, "bevelMC", f);
      testAngleSetter(f);
      testDistanceSetter(f);
      testColorAndAlphaSetters(f, "highlightColor", "highlightAlpha");
      testColorAndAlphaSetters(f, "shadowColor", "shadowAlpha");
      testQualitySetter(f);
      testStrengthSetter(f);
      testBlurSetters(f);
      testTypeSetter(f);
      testBooleanSetter(f, "knockout");
    }

    trace("");

    {
      var f = constructAndTestClone("BlurFilter");
      testMovieClipFilterSetter(current, "blurMC", f);
      testQualitySetter(f);
      testBlurSetters(f);
    }

    trace("");

    {
      var f = constructAndTestClone("ColorMatrixFilter");
      testMovieClipFilterSetter(current, "colorMatrixMC", f);

      var fClass = flash.filters.ColorMatrixFilter;
      trace("// new ColorMatrixFilter(null)");
      traceAllProps(new fClass(null));
      trace("// new ColorMatrixFilter(undefined)");
      traceAllProps(new fClass(undefined));
      trace("// new ColorMatrixFilter(-1)");
      traceAllProps(new fClass(-1));

      setAndTraceProp(f, "matrix", []);
      setAndTraceProp(f, "matrix", [0]);
      setAndTraceProp(f, "matrix", [1.5]);
      setAndTraceProp(f, "matrix", [1000]);
      setAndTraceProp(f, "matrix", [-1000]);
      setAndTraceProp(f, "matrix", [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20]);

      setAndTraceProp(f, "matrix", null);
      setAndTraceProp(f, "matrix", undefined);
      setAndTraceProp(f, "matrix", [null, null]);
      setAndTraceProp(f, "matrix", ["test", undefined]);
      setAndTraceProp(f, "matrix", [true, false]);
      setAndTraceProp(f, "matrix", "ASDASDASD");
      setAndTraceProp(f, "matrix", -1);
    }

    trace("");

    {
      var f = constructAndTestClone("ConvolutionFilter");
      testMovieClipFilterSetter(current, "convolutionMC", f);
      testColorAndAlphaSetters(f, "color", "alpha");

      setAndTraceProp(f, "bias", 100.5);
      setAndTraceProp(f, "bias", -100.5);
      setAndTraceProp(f, "divisor", 100.5);
      setAndTraceProp(f, "divisor", -100.5);
      testBooleanSetter(f, "preserveAlpha");
      testBooleanSetter(f, "clamp");

      var opts = { trace: ["matrixX", "matrixY", "matrix"] };
      setAndTraceProp(f, "matrix", [1, 2, 3, 4], opts);
      setAndTraceProp(f, "matrix", "ASDASD", opts);
      setAndTraceProp(f, "matrix", { length: -1 }, opts);
      setAndTraceProp(f, "matrix", [-1000.5], opts);

      setAndTraceProp(f, "matrixX", 2, opts);
      setAndTraceProp(f, "matrixY", 2, opts);
      setAndTraceProp(f, "matrix", "ASDASDASD", opts);
      setAndTraceProp(f, "matrix", 1234, opts);

      setAndTraceProp(f, "matrix", [-1, -2, -3, -4, -5], opts);
      setAndTraceProp(f, "matrixX", 3.5, opts);
      setAndTraceProp(f, "matrixY", 4.5, opts);
      setAndTraceProp(f, "matrix", "ASD", opts);
      setAndTraceProp(f, "matrix", ["1aaa", "2", null, undefined, 5], opts);

      trace("// f.matrixX = f.matrixY = -100");
      f.matrixX = f.matrixY = -100;
      traceProps(f, opts.trace);

      trace("// f.matrixX = f.matrixY = 100");
      f.matrixX = f.matrixY = 100;
      traceProps(f, opts.trace);
    }

    trace("");

    {
      // NOTE: DisplacementMapFilters can't be embedded in SWFs.
      var f = constructAndTestClone("DisplacementMapFilter");
      traceAllProps(f);
      testColorAndAlphaSetters(f, "color", "alpha");

      trace("// f.componentX = f.componentY = 123");
      f.componentX = f.componentY = 123;
      traceProps(f, ["componentX", "componentY"]);

      trace("// f.componentX = f.componentY = -234.5");
      f.componentX = f.componentY = -234.5;
      traceProps(f, ["componentX", "componentY"]);

      trace("// f.scaleX = f.scaleY = -234.5");
      f.scaleX = f.scaleY = -234.5;
      traceProps(f, ["scaleX", "scaleY"]);

      trace("// f.scaleX = f.scaleY = 65536");
      f.scaleX = f.scaleY = 65536;
      traceProps(f, ["scaleX", "scaleY"]);

      trace("// f.scaleX = f.scaleY = -65536");
      f.scaleX = f.scaleY = -65536;
      traceProps(f, ["scaleX", "scaleY"]);

      setAndTraceProp(f, "mode", "clamp");
      setAndTraceProp(f, "mode", "test");
      setAndTraceProp(f, "mode", "ignore");
      setAndTraceProp(f, "mode", null);
      setAndTraceProp(f, "mode", "color");
      setAndTraceProp(f, "mode", "wrap");
      setAndTraceProp(f, "mode", "Color");

      var bm = new flash.display.BitmapData(10, 10);
      setAndTraceProp(f, "mapBitmap", bm);

      trace("// f.mapBitmap.setPixel32(0, 0, 0xFF0000FF)");
      f.mapBitmap = bm;
      bm.setPixel(0, 0, 0xFF0000FF);
      trace("width = " + f.mapBitmap.width
        + ", height = " + f.mapBitmap.height
        + ", getPixel(0, 0) = " + f.mapBitmap.getPixel32(0, 0)
      );

      trace ("// f.mapBitmap.dispose()");
      bm.dispose();
      trace("width = " + f.mapBitmap.width + ", height = " + f.mapBitmap.height);
      setAndTraceProp(f, "mapBitmap", 45);

      setAndTraceProp(f, "mapPoint", new flash.geom.Point(12.5, -4200));
      setAndTraceProp(f, "mapPoint", { x: 3 });
      setAndTraceProp(f, "mapPoint", { x: 65540, y: -65540 });
      setAndTraceProp(f, "mapPoint", null);
      trace("// f.mapPoint == f.mapPoint");
      trace(f.mapPoint == f.mapPoint);
      trace("// f.mapPoint.x += 10");
      f.mapPoint.x += 10;
      trace("mapPoint=" + f.mapPoint);
    }

    trace("");

    {
      var f = constructAndTestClone("DropShadowFilter");
      testMovieClipFilterSetter(current, "dropShadowMC", f);
      testDistanceSetter(f);
      testAngleSetter(f);
      testColorAndAlphaSetters(f, "color", "alpha");
      testQualitySetter(f);
      testStrengthSetter(f);
      testBlurSetters(f);
      testBooleanSetter(f, "inner");
      testBooleanSetter(f, "knockout");
      testBooleanSetter(f, "hideObject");
    }

    trace("");

    {
      var f = constructAndTestClone("GlowFilter");
      testMovieClipFilterSetter(current, "glowMC", f);
      testColorAndAlphaSetters(f, "color", "alpha");
      testQualitySetter(f);
      testStrengthSetter(f);
      testBlurSetters(f);
      testBooleanSetter(f, "inner");
      testBooleanSetter(f, "knockout");
    }

    trace("");

    {
      var f = constructAndTestClone("GradientBevelFilter");
      testMovieClipFilterSetter(current, "gradientBevelMC", f);
      testDistanceSetter(f);
      testAngleSetter(f);
      testGradientArraySetters(f);
      testBlurSetters(f);
      testQualitySetter(f);
      testStrengthSetter(f);
      testBooleanSetter(f, "knockout");
      testTypeSetter(f);
    }

    trace("");

    {
      var f = constructAndTestClone("GradientGlowFilter");
      testMovieClipFilterSetter(current, "gradientGlowMC", f);
      testDistanceSetter(f);
      testAngleSetter(f);
      testGradientArraySetters(f);
      testBlurSetters(f);
      testQualitySetter(f);
      testStrengthSetter(f);
      testBooleanSetter(f, "knockout");
      testTypeSetter(f);
    }
  }

  /**** SHARED TESTS ****/

  static function constructAndTestClone(className) {
    trace("// new " + className);
    var f = new _global.flash.filters[className]();
    trace(f);

    trace("// f.clone()");
    var cloned = f.clone();
    trace(cloned);

    trace("// f == f.clone()");
    trace(f == cloned);

    return f;
  }

  static function testMovieClipFilterSetter(current, clipName, f) {
    var clip = current[clipName];

    trace("// " + clipName + ".filters[0]");
    traceAllProps(clip.filters[0]);

    trace("// " + clipName + ".filters = [f]");
    clip.filters = [f];
    traceAllProps(clip.filters[0]);
  }

  static function testDistanceSetter(f) {
    setAndTraceProp(f, "distance", 1000);
    setAndTraceProp(f, "distance", 2.5);
    setAndTraceProp(f, "distance", -1);
  }

  static function testAngleSetter(f) {
    setAndTraceProp(f, "angle", 360);
    setAndTraceProp(f, "angle", 361);
    setAndTraceProp(f, "angle", -1);
    setAndTraceProp(f, "angle", 366);
  }

  static function testBlurSetters(f) {
    trace("// f.blurX = f.blurY = 100.5");
    f.blurX = f.blurY = 100.5;
    traceProps(f, ["blurX", "blurY"]);

    trace("// f.blurX = f.blurY = -1");
    f.blurX = f.blurY = -1;
    traceProps(f, ["blurX", "blurY"]);

    trace("// f.blurX = f.blurY = 256");
    f.blurX = f.blurY = 256;
    traceProps(f, ["blurX", "blurY"]);
  }

  static function testQualitySetter(f) {
    setAndTraceProp(f, "quality", 2.5);
    setAndTraceProp(f, "quality", -1);
    setAndTraceProp(f, "quality", 100);
  }

  static function testStrengthSetter(f) {
    setAndTraceProp(f, "strength", 256);
    setAndTraceProp(f, "strength", 1.5);
    setAndTraceProp(f, "strength", -1);
  }

  static function testBooleanSetter(f, name) {
    setAndTraceProp(f, name, false);
    setAndTraceProp(f, name, true);
    setAndTraceProp(f, name, null);
  }

  static function testTypeSetter(f) {
    f.type = "outer";
    setAndTraceProp(f, "type", "invalid");
    f.type = "outer";
    setAndTraceProp(f, "type", "INNER");
    f.type = "outer";
    setAndTraceProp(f, "type", 0);
  }

  static function testColorAndAlphaSetters(f, colorName, alphaName) {
    var opts = { trace: [colorName, alphaName] };

    // Order is important, to make sure color and alpha are independent
    setAndTraceProp(f, alphaName, 0.5, opts);
    setAndTraceProp(f, colorName, 0x10000FF, opts);
    setAndTraceProp(f, alphaName, 1.5, opts);
    setAndTraceProp(f, colorName, -0x12345678, opts);
    setAndTraceProp(f, alphaName, -1, opts);
  }

  static function testGradientArraySetters(f) {
    var opts = { trace: ["colors", "alphas", "ratios"] };

    setAndTraceProp(f, "alphas", [1], opts);
    setAndTraceProp(f, "colors", [-0x12345678, 10], opts);
    setAndTraceProp(f, "ratios", [500, 50.5, 100], opts);
    setAndTraceProp(f, "colors", "ASDF", opts);
    setAndTraceProp(f, "ratios", null, opts);
    setAndTraceProp(f, "alphas", [0.5, 1.5, -0.5], opts);
    setAndTraceProp(f, "ratios", [50], opts);
    setAndTraceProp(f, "colors", 5, opts);
    setAndTraceProp(f, "colors", [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17], opts);
  }
  
  /**** HELPERS ****/

  static function valueToString(v) {
    return v instanceof Array ? ("[" + v + "]") : ("" + v);
  }

  static function setAndTraceProp(f, propName, propValue, options) {
    trace("// f." + propName + " = " + valueToString(propValue));
    f[propName] = propValue;

    if (options.trace === undefined) {
      trace(propName + "=" + valueToString(f[propName]));
    } else {
      traceProps(f, options.trace);
    }
  }

  static function traceProps(f , props) {
    if (f == null) {
      trace(f);
      return;
    }

    var str = "";
    for (var i = 0; i < props.length; i++) {
      var prop = props[i];
      if (str != "") str += ", ";
      str += prop + "=" + valueToString(f[prop]);
    }
    trace(str);
  }

  static function traceAllProps(f) {
    if (f == null) {
      trace(f);
      return;
    }

    var props = [];
    for (var prop in f) {
      props.push(prop);
    }

    // Flash returns properties in reverse order.
    props.reverse();
    traceProps(f, props);
  }
}
