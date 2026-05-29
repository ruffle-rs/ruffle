var s1 = new Sound("mc");

function traceProps() {
   trace("// s1.getVolume()");
   trace(s1.getVolume());
   trace("// s1.getPan()");
   trace(s1.getPan());
   trace("// s1.getTransform()");
   trace(s1.getTransform());
   trace("// s1.getDuration()");
   trace(s1.getDuration());
   trace("// s1.getPosition()");
   trace(s1.getPosition());
   trace("// s1.duration");
   trace(s1.duration);
   trace("// s1.position");
   trace(s1.position);
   trace("");
}

traceProps();

createEmptyMovieClip("mc",1);
trace("created mc");
trace("");
traceProps();

mc.removeMovieClip();
trace("removed mc");
trace("");
traceProps();

createEmptyMovieClip("mc2",2);
trace("created new mc, sound will be made with MCR");
trace("");
var s1 = new Sound(mc2);
s1.setVolume(50);
traceProps();

mc2.removeMovieClip();
trace("removed mc2");
trace("");
traceProps();

createEmptyMovieClip("mc2",2);
trace("recreated mc2");
trace("");
traceProps();
