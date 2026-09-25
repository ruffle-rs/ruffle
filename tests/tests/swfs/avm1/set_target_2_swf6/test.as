createEmptyMovieClip("clip1", 1);
createEmptyMovieClip("clip2", 2);
createEmptyMovieClip("clip3", 3);
createEmptyMovieClip("5", 4);

var obj1 = {};
obj1.toString = function() {
	trace("obj1.toString");
	return "clip1";
};

var obj2 = {};
obj2.valueOf = function() {
	trace("obj2.valueOf");
	return "clip2";
};

var obj3 = "clip3";
obj3.toString = function() {
	trace("obj3.toString");
	return "clip2";
};

tellTarget(obj1) {
	trace("tellTarget(obj1):" + _name);
}

tellTarget(obj2) {
	trace("tellTarget(obj2):" + _name);
}

tellTarget(obj3) {
	trace("tellTarget(obj3):" + _name);
}

tellTarget(null) {
	trace("tellTarget(null):" + _name);
}

tellTarget(undefined) {
	trace("tellTarget(undefined):" + _name);
}

tellTarget(5) {
	trace("tellTarget(5):" + _name);
}

tellTarget(4) {
	trace("tellTarget(4):" + _name);
}

createEmptyMovieClip("clip4", 5);
clip4.toString = function() {
	trace("clip4.toString");
	return "clip2";
};

tellTarget(clip4) {
	trace("tellTarget(clip4):" + _name);
}

var a = createEmptyMovieClip("clip5", 6);
a.x = "a";
var b = createEmptyMovieClip("clip5", 7);
b.x = "b";

tellTarget(a) {
	trace("tellTarget(a):" + x);
}

tellTarget(b) {
	trace("tellTarget(b):" + x);
}

tellTarget(clip5) {
	trace("tellTarget(clip5):" + x);
}

tellTarget("clip5") {
	trace("tellTarget(\"clip5\"):" + x);
}
