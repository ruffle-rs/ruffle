// Flash Player doesn't define width and height on Video.prototype up front.
// When a Video display object is created, they are defined on its prototype,
// but at most once per prototype object, regardless of what it contains:
// existing values are overwritten, and once deleted, they are never defined
// on that prototype again. `new Video()` doesn't count.
//
// This is independent of the native data of the prototype object: native
// objects get them as well, and a prototype that got them can still be made
// native, without getting them again. `makeDate` tries to initialize a Date
// over the object, which only works if it isn't native yet (or is already a
// Date).
//
// test.swf (from test.swfasm) places a Video as `v2` on frame 2. Going to frame 1 and back to
// frame 2 creates a new Video each time.
var FAKE_DATE_PROTO = {__proto__:Date.prototype, __constructor__:Date, __initializeNative:function()
{
   super();
}};
function makeDate(o)
{
   var original = o.__proto__;
   o.__proto__ = FAKE_DATE_PROTO;
   o.__initializeNative();
   var isDate = typeof Date.prototype.getDate.call(o) == "number";
   o.__proto__ = original;
   return isDate;
}
function placeWith(proto)
{
   Video.prototype = proto;
   _root.gotoAndStop(1);
   _root.gotoAndStop(2);
   return _root.v2;
}
function desc(o, name)
{
   return !o.hasOwnProperty(name) ? "-" : typeof o[name];
}
function has(o)
{
   return "width=" + desc(o,"width") + " height=" + desc(o,"height");
}
function place(label, proto)
{
   _root.gotoAndStop(1);
   _root.gotoAndStop(2);
   var v = _root.v2;
   trace(label + ": " + has(proto) + ", v.__proto__ == proto: " + (v.__proto__ == proto) + ", v.width=" + v.width + " v.height=" + v.height);
}
if(this.started == undefined)
{
   this.started = true;
   stop();
   var orig = Video.prototype;
   trace("original prototype: " + has(orig));
   var nv = new Video();
   trace("after new Video(): " + has(orig));
   this.onEnterFrame = function()
   {
      delete this.onEnterFrame;
      var p1 = {marker:1};
      p1.width = 5;
      Video.prototype = p1;
      place("first Video, p1 had width=5", p1);
      trace("original prototype: " + has(orig));
      // Flash Player also defines smoothing and deblocking here, which
      // this test doesn't cover.
      var e = [];
      for(var k in p1)
      {
         if(k == "width" || k == "height" || k == "marker")
         {
            e.push(k);
         }
      }
      e.sort();
      trace("enumerable: " + e.join(","));
      delete p1.width;
      place("p1 without width", p1);
      delete p1.height;
      place("p1 without width and height", p1);
      var nv2 = new Video();
      trace("after new Video(): " + has(p1));
      var p2 = {};
      Video.prototype = p2;
      var nv3 = new Video();
      trace("swapped to p2, after new Video(): " + has(p2));
      place("swapped to p2", p2);
      Video.prototype = p1;
      place("swapped back to p1", p1);
      var p3 = {height:1};
      Video.prototype = p3;
      place("swapped to p3 = {height: 1}", p3);

      // Plain object, got them, then made a Date.
      var n1 = {};
      placeWith(n1);
      trace("n1 = {}, after placing a Video: " + has(n1));
      trace("n1, made a Date: " + makeDate(n1));
      delete n1.width;
      delete n1.height;
      placeWith(n1);
      trace("n1, deleted width and height, after placing a Video: " + has(n1));
      // Plain object, made a Date, then used as prototype.
      var n2 = {};
      trace("n2 = {}, made a Date: " + makeDate(n2));
      placeWith(n2);
      trace("n2, after placing a Video: " + has(n2));
      // Native objects as prototype.
      var a = [1,2];
      var v = placeWith(a);
      a[5] = 0;
      trace("[1, 2], after placing a Video: " + has(a) + ", length=" + a.length + ", v.width=" + v.width);
      var tf = new TextFormat();
      tf.font = "Foo";
      v = placeWith(tf);
      tf.size = 12;
      trace("new TextFormat(), after placing a Video: " + has(tf) + ", font=" + tf.font + ", size=" + tf.size + ", v.width=" + v.width);
      var d = new Date(2000,0,1);
      v = placeWith(d);
      trace("new Date(), after placing a Video: " + has(d) + ", year=" + d.getFullYear() + ", v.width=" + v.width);
      trace("done");
   };
}
