// Companion test to button_textfield_path_reference (ruffle-rs/ruffle#2583).
//
// Checks that an AVM1 reference to a display object keeps following its path
// when the timeline replaces the object at that path with one of a *different*
// type, and that `typeof` is re-evaluated against whatever is currently there
// (i.e. nothing is cached from the object the reference was first taken from).
//
// No .fla is available: test.swf was generated with a small program using
// Ruffle's own `swf` crate (SWF version 8, 550x400, 24 fps). output.txt was
// recorded from the Flash Player 10.0 r12 standalone debugger.
//
// Timeline (root):
//   Frame 1: PlaceObject2 DefineSprite #3     depth 1 name "obj"   (MovieClip)
//   Frame 2: RemoveObject2 depth 1
//            PlaceObject2 DefineButton2 #4    depth 1 name "obj"   (Button)
//   Frame 3: RemoveObject2 depth 1
//            PlaceObject2 DefineEditText #5   depth 1 name "obj"   (TextField)
//   Frame 4: RemoveObject2 depth 1
//            PlaceObject2 DefineSprite #3     depth 1 name "obj"   (MovieClip)
//   Frame 5: RemoveObject2 depth 1                                 (nothing)
//   Frame 6: PlaceObject2 DefineButton2 #4    depth 1 name "obj"   (Button)
//
// `r`, `rb` and `rt` are captured while "obj" is a MovieClip, a Button and a
// TextField respectively. The frame scripts below are what each DoAction does.

// --- Frame 1 (MovieClip) ---
var r = obj;
trace("f1 MovieClip: typeof obj: " + typeof obj);
trace("f1 MovieClip: typeof r: " + typeof r);
trace("f1 MovieClip: r: " + r);
trace("f1 MovieClip: r == obj: " + (r == obj));

// --- Frame 2 (MovieClip -> Button) ---
trace("f2 Button: typeof obj: " + typeof obj);
trace("f2 Button: typeof r: " + typeof r);
trace("f2 Button: r: " + r);
trace("f2 Button: r == obj: " + (r == obj));
var rb = obj;
trace("f2 Button: typeof rb: " + typeof rb);

// --- Frame 3 (Button -> TextField) ---
trace("f3 TextField: typeof obj: " + typeof obj);
trace("f3 TextField: typeof r: " + typeof r);
trace("f3 TextField: typeof rb: " + typeof rb);
trace("f3 TextField: r: " + r);
trace("f3 TextField: r == obj: " + (r == obj));
trace("f3 TextField: rb == obj: " + (rb == obj));
var rt = obj;
trace("f3 TextField: typeof rt: " + typeof rt);

// --- Frame 4 (TextField -> MovieClip) ---
trace("f4 MovieClip: typeof obj: " + typeof obj);
trace("f4 MovieClip: typeof r: " + typeof r);
trace("f4 MovieClip: typeof rb: " + typeof rb);
trace("f4 MovieClip: typeof rt: " + typeof rt);
trace("f4 MovieClip: rb: " + rb);
trace("f4 MovieClip: r == obj: " + (r == obj));
trace("f4 MovieClip: rb == obj: " + (rb == obj));
trace("f4 MovieClip: rt == obj: " + (rt == obj));

// --- Frame 5 (removed, nothing at the path) ---
trace("f5 removed: typeof obj: " + typeof obj);
trace("f5 removed: typeof r: " + typeof r);
trace("f5 removed: typeof rb: " + typeof rb);
trace("f5 removed: typeof rt: " + typeof rt);
trace("f5 removed: r: " + r);
trace("f5 removed: rb: " + rb);
trace("f5 removed: rt: " + rt);

// --- Frame 6 (Button again) ---
trace("f6 Button: typeof obj: " + typeof obj);
trace("f6 Button: typeof r: " + typeof r);
trace("f6 Button: typeof rb: " + typeof rb);
trace("f6 Button: typeof rt: " + typeof rt);
trace("f6 Button: rt: " + rt);
trace("f6 Button: r == obj: " + (r == obj));
trace("f6 Button: rb == obj: " + (rb == obj));
trace("f6 Button: rt == obj: " + (rt == obj));
stop();
