// attachMovie and duplicateMovieClip with a movie clip as the init object.
// The new clip gets a copy of the init object's properties.

createEmptyMovieClip("src", 1);
src.foo = "first";

trace("// A reference to an existing clip");
var current = src;
attachMovie("symbol", "attached1", 10, current);
trace("attachMovie: " + attached1.foo);
attached1.duplicateMovieClip("duplicated1", 12, current);
trace("duplicateMovieClip: " + duplicated1.foo);
trace("");

trace("// A reference to a removed clip, so it refers to the clip at its path");
var removed = src;
src.removeMovieClip();
createEmptyMovieClip("src", 1);
src.foo = "second";
attachMovie("symbol", "attached2", 11, removed);
trace("attachMovie: " + attached2.foo);
attached1.duplicateMovieClip("duplicated2", 13, removed);
trace("duplicateMovieClip: " + duplicated2.foo);
trace("");

trace("// A plain object");
attachMovie("symbol", "attached3", 14, {foo: "plain"});
trace("attachMovie: " + attached3.foo);
