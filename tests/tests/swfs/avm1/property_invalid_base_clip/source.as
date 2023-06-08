mc1 = _root.createEmptyMovieClip("mc1", _root.getNextHighestDepth());
mc2 = _root.createEmptyMovieClip("mc2", _root.getNextHighestDepth());
mc3 = _root.createEmptyMovieClip("mc3", _root.getNextHighestDepth());
_global.clip3 = mc3;

tellTarget(mc2) {
	_root.mc3._x = 123;
	_root.mc3._y = 456;

	trace("getProperty('_root.mc3', _X)");
	trace(getProperty("_root.mc3", _X));

	trace("getProperty('mc3', _X)");
	trace(getProperty("mc3", _X));

	trace("getProperty('clip3', _X)");
	trace(getProperty("clip3", _X));
	trace("");

	trace("setProperty('_root.mc3', _Y, 789)");
	trace(setProperty("_root.mc3", _Y, 789));
	trace(getProperty("_root.mc3", _Y));

	trace("setProperty('mc3', _Y, 123)");
	trace(setProperty("mc3", _Y, 123));
	trace(getProperty("mc3", _Y));

	trace("setProperty('clip3', _Y, 456)");
	trace(setProperty("clip3", _Y, 456));
	trace(getProperty("clip3", _Y));
	trace("");

	tellTarget(mc1) {
		mc2.removeMovieClip();
	}

	_root.mc3._x = 123;
	_root.mc3._y = 456;

	trace("// tellTarget(mc1) { mc2.removeMovieClip() }");
	trace("");

	trace("getProperty('_root.mc3', _X)");
	trace(getProperty("_root.mc3", _X));

	trace("getProperty('mc3', _X)");
	trace(getProperty("mc3", _X));

	trace("getProperty('clip3', _X)");
	trace(getProperty("clip3", _X));
	trace("");

	trace("setProperty('_root.mc3', _Y, 789)");
	trace(setProperty("_root.mc3", _Y, 789));
	trace(getProperty("_root.mc3", _Y));

	trace("setProperty('mc3', _Y, 123)");
	trace(setProperty("mc3", _Y, 123));
	trace(getProperty("mc3", _Y));

	trace("setProperty('clip3', _Y, 456)");
	trace(setProperty("clip3", _Y, 456));
	trace(getProperty("clip3", _Y));
	trace("");
}
