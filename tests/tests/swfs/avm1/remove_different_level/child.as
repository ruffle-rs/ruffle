createEmptyMovieClip("mc", 1);

mc.onUnload = function() {
   trace("onUnload");
};

mc.removeMovieClip();
trace(mc);

this.onEnterFrame = function() {
   delete this.onEnterFrame;
   trace(mc);
};
