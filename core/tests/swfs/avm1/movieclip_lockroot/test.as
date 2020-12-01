var levels = 1;

function testSet(value) {
	var valueLabel = typeof value == "string" ? "\"" + value + "\"" : value;
	this._lockroot = false;
	this._lockroot = value;
	if (this._lockroot) {
		trace("Parent: setting _lockroot to " + valueLabel + " sets it to true.");
		this._lockroot = false;
		return;
	}
	this._lockroot = true;
	this._lockroot = value;
	if (!this._lockroot) {
		trace("Parent: setting _lockroot to " + valueLabel + " sets it to false.");
		return;
	}
	
	this._lockroot = false;
	trace("Parent: setting _lockroot to " + valueLabel + " has no effect.");
}

function testSelf() {
	// Prints _level0, _level0, false
	trace("Parent: level is " + this + ", root is " + _root + ", _lockroot is " + this._lockroot);
	
	// Edge case tests
	testSet(true); /// true
	testSet(false); // false
	testSet(undefined); //false
	testSet(null); // false
	testSet(NaN); // false
	testSet(-1); // true
	testSet(0); // false
	testSet(1); // true
	testSet(1337); // true
	testSet("true"); // true
	testSet("false"); // true
	testSet("foo"); // true
	testSet(""); // false
	testSet("0"); // true
	testSet({}); // true
}

function testChildren() {
	// 1 = normal child, lockroot unset
	// 2 = lockroot set on container before loading (must inherit)
	// 3 = lockroot set by child function
	
	// Top
	createEmptyMovieClip("child1", levels++);
	child1.loadMovie("child.swf");
	createEmptyMovieClip("child2", levels++);
	child2._lockroot = true;
	child2.loadMovie("child.swf");
	createEmptyMovieClip("child3", levels++);
	child3.loadMovie("child.swf");
	
	// Wait for initial loads
	setTimeout(createGrandChildren, 50);
	
	// Test it all
	setTimeout(testAllChildren, 250);
}

function createGrandChildren() {
	// Each grandchild
	_root.child1.createEmptyMovieClip("child1", levels++);
	_root.child1.child1.loadMovie("child.swf");
	_root.child1.createEmptyMovieClip("child2", levels++);
	_root.child1.child2._lockroot = true;
	_root.child1.child2.loadMovie("child.swf");
	_root.child1.createEmptyMovieClip("child3", levels++);
	_root.child1.child3.loadMovie("child.swf");

	_root.child2.createEmptyMovieClip("child1", levels++);
	_root.child2.child1.loadMovie("child.swf");
	_root.child2.createEmptyMovieClip("child2", levels++);
	_root.child2.child2._lockroot = true;
	_root.child2.child2.loadMovie("child.swf");
	_root.child2.createEmptyMovieClip("child3", levels++);
	_root.child2.child3.loadMovie("child.swf");

	_root.child3.createEmptyMovieClip("child1", levels++);
	_root.child3.child1.loadMovie("child.swf");
	_root.child3.createEmptyMovieClip("child2", levels++);
	_root.child3.child2._lockroot = true;
	_root.child3.child2.loadMovie("child.swf");
	_root.child3.createEmptyMovieClip("child3", levels++);
	_root.child3.child3.loadMovie("child.swf");
}

function testAllChildren() {
	_root.child1.test(); // Root is _level0, lockroot false
	_root.child2.test(); // Root is self, lockroot true
	_root.child3.setLock(true);
	_root.child3.test(); // Root is self, lockroot true

	_root.child1.child1.test(); // Root is _level0, lockroot false
	_root.child1.child2.test(); // Root is self, lockroot true
	_root.child1.child3.setLock(true);
	_root.child1.child3.test(); // Root is self, lockroot true

	_root.child2.child1.test(); // Root is _parent, lockroot true
	_root.child2.child2.test(); // Root is self, lockroot true
	_root.child2.child3.setLock(true);
	_root.child2.child3.test(); // Root is self, lockroot true

	_root.child3.child1.test(); // Root is _parent, lockroot true
	_root.child3.child2.test(); // Root is self, lockroot true
	_root.child3.child3.setLock(true);
	_root.child3.child3.test(); // Root is self, lockroot true
	
	trace("All tests done.");
}

function test() {
	testSelf();
	testChildren();
}

test();