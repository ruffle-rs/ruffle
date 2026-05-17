var mc = createEmptyMovieClip("mc", 1);
var s1 = new Sound(mc);

function traceProps() {
   trace("// s1.getVolume()");
   trace(s1.getVolume());
   trace("// s1.getPan()");
   trace(s1.getPan());
   trace("// s1.getTransform()");
   var t = s1.getTransform()
   trace(t);
   for (var i in t) {
      trace("  " + i + ": " + t[i]);
   }
   trace("");
}

s1.setVolume(50);
s1.setPan(-25);
var o = new Object();
o.ll = 10
o.lr = 20
o.rr = 30
o.rl = 40
s1.setTransform(o);

traceProps();

mc._name = "changed";

s1.setVolume(10);
s1.setPan(75);
var o = new Object();
o.ll = 50
o.lr = 60
o.rr = 70
o.rl = 80
s1.setTransform(o);

mc._name = "mc";

traceProps();

var mcr = createEmptyMovieClip("mc2", 2);
s1 = new Sound(mcr);

s1.setVolume(50);
s1.setPan(-25);
var o = new Object();
o.ll = 10
o.lr = 20
o.rr = 30
o.rl = 40
s1.setTransform(o);

traceProps();

mcr._name = "changed";

s1.setVolume(10);
s1.setPan(75);
var o = new Object();
o.ll = 50
o.lr = 60
o.rr = 70
o.rl = 80
s1.setTransform(o);

mcr._name = "mc2";

traceProps();

fscommand("quit");
