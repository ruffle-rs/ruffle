var s_mc = new Sound("mc");
var s_mc2 = new Sound("mc");
var s_glob = new Sound();

function traceProps(s, n) {
   trace("// s_" + n + ".getVolume()");
   trace(s.getVolume());
   trace("// s_" + n + ".getPan()");
   trace(s.getPan());
   trace("// s_" + n + ".getTransform()");
   var t = s.getTransform()
   trace(t);
   for (var i in t) {
      trace("  " + i + ": " + t[i]);
   }
   trace("");
}

s_mc.loadSound("invalid.mp3");

traceProps(s_mc, "mc");

createEmptyMovieClip("mc", 1);

s_mc.setVolume(50);
s_mc.setPan(-25);
var o = new Object();
o.ll = 10
o.lr = 20
o.rr = 30
o.rl = 40
s_mc.setTransform(o);

traceProps(s_mc, "mc");

trace("// s_mc.loadSound()");
trace("");
s_mc.loadSound();

traceProps(s_mc, "mc");

trace("// s_mc.loadSound(\"\")");
trace("");
s_mc.loadSound("");

s_mc.setVolume(200);
s_mc.setPan(50);

traceProps(s_mc, "mc");

trace("// s_mc.loadSound(\"invalid.mp3\")");
trace("");
s_mc.loadSound("invalid.mp3");

traceProps(s_mc, "mc");

s_mc.setVolume(1);
s_mc.setPan(-2);
var o = new Object();
o.ll = 3
o.lr = 4
o.rr = 5
o.rl = 6
s_mc.setTransform(o);

traceProps(s_mc, "mc");

traceProps(s_mc2, "mc2");

traceProps(s_glob, "glob");

s_mc2.loadSound("invalid.mp3");

s_mc2.setVolume(10);
s_mc2.setPan(-12);
var o = new Object();
o.ll = 13
o.lr = 14
o.rr = 15
o.rl = 16
s_mc2.setTransform(o);

traceProps(s_mc2, "mc2");
traceProps(s_mc, "mc");

fscommand("quit");
