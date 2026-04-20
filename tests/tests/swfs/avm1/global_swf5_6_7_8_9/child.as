function getGlobal() { return _global; }
this.global = getGlobal();
this.global2 = _global;
this.anObject = {};
this.anArray = [1,2,3];
this.aFunction = function(){};
this.anObjectClass = Object;
this.aMovieClipClass = MovieClip;
this.aBooleanClass = Boolean;
trace("child loaded");
