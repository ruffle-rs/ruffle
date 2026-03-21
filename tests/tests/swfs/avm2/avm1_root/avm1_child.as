var foo = "bar";
trace("avm1_child this: " + this);
trace("_level0: " + this._level0);
trace("_level-61440: " + this["_level-61440"]);
trace("_level0.foo: " + this._level0.foo);
trace("_level-61440.foo: " + this["_level-61440"].foo);

this.createEmptyMovieClip("emptyMC", 1);
trace("emptyMC._target: " + emptyMC._target);

loadMovieNum("level1_child.swf", 1);
