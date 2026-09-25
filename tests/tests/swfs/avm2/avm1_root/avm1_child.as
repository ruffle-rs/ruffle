var foo = "bar";

trace("=== avm1_child");
trace("this: " + this);
trace("_level0: " + _level0);
trace("_root: " + _root);
trace("_parent: " + _parent);
trace("_level-61440: " + this["_level-61440"]);
trace("_level0.foo: " + this._level0.foo);

var child_via_mcl = this.createEmptyMovieClip("child_via_mcl", 1);
trace("child_via_mcl._target: " + child_via_mcl._target);

var child_via_loadmovie = this.createEmptyMovieClip("child_via_loadmovie", 2);

trace("");

loadMovieNum("level1_child.swf", 1);

this._name = "defined_avm1_name";

var mcl = new MovieClipLoader();
mcl.loadClip("other_child.swf", child_via_mcl);

this.onEnterFrame = function() {
    delete this.onEnterFrame;
    loadMovie("other_child.swf", child_via_loadmovie);
}
