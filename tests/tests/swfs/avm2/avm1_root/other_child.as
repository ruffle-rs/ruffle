trace("=== other_child.swf (" + this._name + ")");
trace("this: " + this);
trace("this._target: " + this._target);
trace("_level0: " + _level0);
trace("_root: " + _root);
trace("_parent: " + _parent);
trace("_parent._parent: " + _parent._parent);
trace("_level0.foo: " + _level0.foo);
trace("_level0.as3Loader: " + _level0.as3Loader);
trace("_level0.as3Loader.defined_avm1_name: " + _level0.as3Loader.defined_avm1_name);
trace("typeof _level0.as3Loader: " + typeof _level0.as3Loader);
trace("_level0.as3Loader._target: " + _level0.as3Loader._target);

_level0.createEmptyMovieClip("level0Clip", 20);
_level0.as3Loader.createEmptyMovieClip("as3LoaderClip", 20);
trace("_level0.level0Clip: " + _level0.level0Clip);
trace("_level0.as3Loader.as3LoaderClip: " + _level0.as3Loader.as3LoaderClip);
trace("typeof _level0.level0Clip: " + typeof _level0.level0Clip);
trace("typeof _level0.as3Loader.as3LoaderClip: " + typeof _level0.as3Loader.as3LoaderClip);

_level0.as3Loader._x = 50;
_level0.level0Clip._x = 25;
_level0.as3Loader.defined_avm1_name._x = 10;

trace("_level0.as3Loader.defined_avm1_name._x: " + _level0.as3Loader.defined_avm1_name._x);

trace("");
