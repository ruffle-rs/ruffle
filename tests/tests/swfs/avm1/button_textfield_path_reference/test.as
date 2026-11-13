// Regression test for ruffle-rs/ruffle#2583.
//
// No .fla is available: test.swf was assembled with JPEXS FFDec (xml2swf for the
// timeline, importScript for the frame scripts below). SWF version 8, 550x400, 24 fps.
//
// Timeline (root):
//   Frame 1: PlaceObject2 DefineButton2 #2   depth 1 name "btn" at (100, 100)
//            PlaceObject2 DefineEditText #4  depth 2 name "txt" at (100, 200)
//   Frame 2: RemoveObject2 depth 1, RemoveObject2 depth 2
//            PlaceObject2 DefineButton2 #3   depth 1 name "btn" at (300, 100)
//            PlaceObject2 DefineEditText #5  depth 2 name "txt" at (300, 200)
//
// Flash stores references to Buttons and TextFields as paths, so `b` and `t`,
// captured in frame 1, resolve to the re-created objects in frame 2.
// input.json then clicks the frame-2 button at (300, 100).

// --- Frame 1 ---
var b = btn;
var t = txt;
trace("frame1 typeof b: " + typeof b);
trace("frame1 b: " + b);
trace("frame1 b._x: " + b._x);
trace("frame1 b == btn: " + (b == btn));
trace("frame1 typeof t: " + typeof t);
trace("frame1 t: " + t);
trace("frame1 t._x: " + t._x);
trace("frame1 t == txt: " + (t == txt));

// --- Frame 2 ---
trace("frame2 typeof b: " + typeof b);
trace("frame2 b: " + b);
trace("frame2 b._x: " + b._x);
trace("frame2 b == btn: " + (b == btn));
trace("frame2 typeof t: " + typeof t);
trace("frame2 t: " + t);
trace("frame2 t._x: " + t._x);
trace("frame2 t == txt: " + (t == txt));
b.onRelease = function()
{
   trace("onRelease fired on b._x = " + b._x);
};
stop();
