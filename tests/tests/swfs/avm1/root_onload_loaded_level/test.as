this.onLoad = function() {
   trace("initial root onLoad");
};

var step = 0;
this.onEnterFrame = function() {
   step++;
   if (step == 3) {
      loadMovieNum("child1.swf", 1);
   } else if (step == 8) {
      _level1.loadMovie("child2.swf");
   } else if (step == 13) {
      loadMovieNum("child3.swf", 0);
   }
};
