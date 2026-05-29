var o = new Object();
o.toString = function() {
   trace("o.toString()");
   return "mc";
};

var mcr = createEmptyMovieClip("mc", 1);

trace("// new Sound(o)");
var s1 = new Sound(o);

trace("// s1.setVolume(50)");
s1.setVolume(50);

trace("// s1.getVolume()");
trace(s1.getVolume());

var s2 = new Sound(mc);
s2.setVolume(25);

trace("// s2.getVolume()");
trace(s2.getVolume());

mcr._name = "changed";

trace("// s2.setVolume(5)");
s2.setVolume(5)

trace("// s2.getVolume()");
trace(s2.getVolume());

mcr._name = "mc";

trace("// s2.getVolume()");
trace(s2.getVolume());

fscommand("quit");
