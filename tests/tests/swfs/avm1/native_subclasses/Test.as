// Compile with:
//  mtasc -main -header 200:150:30 Test.as -swf test.swf -version 8 
class Test {

  static function main(current) {
    var obj;

    obj = newSubclassOf("Boolean", true);
    trace("obj: " + obj);
    trace("");

    obj = newSubclassOf("Number", 123.4);
    trace("obj: " + obj);
    trace("");

    obj = newSubclassOf("String", "hello");
    trace("obj.length: " + obj.length);
    trace("");

    obj = newSubclassOf("Array", "foo", "bar");
    trace("obj.length: " + obj.length);
    trace("obj.shift(): " + obj.shift());
    trace("obj.length: " + obj.length);
    trace("obj.shift(): " + obj.shift());
    trace("obj.length: " + obj.length);
    trace("obj.shift(): " + obj.shift());
    trace("obj.length: " + obj.length);
    trace("");

    obj = newSubclassOf("Function", "myFunc");
    trace("obj: " + obj);
    trace("");

    obj = newSubclassOf("Date", 123456);
    trace("obj.getTime(): " + obj.getTime());
    trace("");

    obj = newSubclassOf("flash.filters.BlurFilter", 10, 20);
    trace("obj.blurX: " + obj.blurX);
    trace("obj.blurY: " + obj.blurY);
    trace("");

    obj = newSubclassOf("flash.filters.BevelFilter", 5, 60);
    trace("obj.distance: " + obj.distance);
    trace("obj.angle: " + obj.angle);
    trace("");

    obj = newSubclassOf("flash.filters.GlowFilter", 0x00FF00, 1);
    trace("obj.color: " + obj.color);
    trace("obj.alpha: " + obj.alpha);
    trace("");

    obj = newSubclassOf("flash.filters.DropShadowFilter", 5, 60);
    trace("obj.distance: " + obj.distance);
    trace("obj.angle: " + obj.angle);
    trace("");

    obj = newSubclassOf("flash.filters.ColorMatrixFilter", [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20]);
    trace("obj.matrix: " + obj.matrix);
    trace("");

    obj = newSubclassOf("flash.filters.DisplacementMapFilter", null, null);
    trace("obj.mapBitmap: " + obj.mapBitmap);
    trace("obj.mapPoint: " + obj.mapPoint);
    trace("");

    obj = newSubclassOf("flash.filters.ConvolutionFilter", 2, 3, [1, 2, 3, 4, 5, 6]);
    trace("obj.matrixX: " + obj.matrixX);
    trace("obj.matrixY: " + obj.matrixY);
    trace("obj.matrix: " + obj.matrix);
    trace("");

    obj = newSubclassOf("flash.filters.GradientBevelFilter", 5, 60);
    trace("obj.distance: " + obj.distance);
    trace("obj.angle: " + obj.angle);
    trace("");

    obj = newSubclassOf("flash.filters.GradientGlowFilter", 5, 60);
    trace("obj.distance: " + obj.distance);
    trace("obj.angle: " + obj.angle);
    trace("");

    obj = newSubclassOf("flash.geom.ColorTransform");
    trace("obj.toString(): " + obj.toString());
    trace("");

    obj = newSubclassOf("flash.geom.Transform", current);
    trace("");

    obj = newSubclassOf("TextFormat", "Arial", 12);
    trace("obj.font: " + obj.font);
    trace("obj.size: " + obj.size);
    trace("");

    obj = newSubclassOf("flash.display.BitmapData", 20, 30);
    trace("obj.width: " + obj.width);
    trace("obj.height: " + obj.height);
    trace("");

    obj = newSubclassOf("XML", "<node />");
    trace("obj.status: " + obj.status);
    trace("");

    obj = newSubclassOf("XMLNode", 1 /* ELEMENT_NODE */, "node");
    trace("obj.nodeType: " + obj.nodeType);
    trace("obj.nodeName: " + obj.nodeName);
    trace("");

    obj = newSubclassOf("LocalConnection");
    trace("");

    obj = newSubclassOf("Sound", current);
    trace("obj.getVolume(): " + obj.getVolume());
    trace("");

    obj = newSubclassOf("TextField.StyleSheet");
    trace("");

    // These aren't native constructors in Flash Player.

    obj = newSubclassOf("NetConnection");
    trace("obj.isConnected: " + obj.isConnected);
    trace("");

    obj = newSubclassOf("NetStream", obj);
    trace("");

    obj = newSubclassOf("XMLSocket");
    trace("");

    obj = newSubclassOf("SharedObject");
    trace("");

    obj = newSubclassOf("flash.net.FileReference");
    trace("");

    obj = newSubclassOf("MovieClip");
    trace("");

    trace("Done!");
  }

  static function newSubclassOf(superClsName /*, args...*/) {
    var superCls = eval(superClsName);
    var args = arguments.slice(1);

    trace("Subclassing " + superClsName + "...");
    var obj = new Test();
    trace("super(" + args + "): " + obj.makeSubclassOf(superCls, args));
    trace("obj.isSubclass: " + obj.isSubclass)
    trace("obj instanceof " + superClsName + ": " + (obj instanceof superCls));
    return obj;
  }

  function makeSubclassOf(superCls, args) {
    // Dynamically decide which class we're extending.
    this.__proto__ = {
      isSubclass: true,
      __proto__: superCls.prototype,
      __constructor__: superCls
    };

    // Work around MTASC's `super` checks.
    var zuper = super;

    // No syntax to call `super()` with variadic arguments, sadly.
    switch (args.length) {
      case 0: return zuper();
      case 1: return zuper(args[0]);
      case 2: return zuper(args[0], args[1]);
      case 3: return zuper(args[0], args[1], args[2]);
      default:
        trace("Unsupported number of args: " + args.length);
        return null;
    }
  }

  function Test() {}
}
