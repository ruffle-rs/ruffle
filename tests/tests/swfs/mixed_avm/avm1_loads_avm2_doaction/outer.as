trace("Hello from outer SWF");
var clip = this.createEmptyMovieClip("clip", 0);
loadMovie("avm2.swf", clip);
trace("Outer SWF: Called loadMovie");
