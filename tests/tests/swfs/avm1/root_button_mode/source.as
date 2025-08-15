// Test made with JPEXS Free Flash Decompiler.

function createMC(name, x, y, parent) {
	var mc = parent.createEmptyMovieClip(name, parent.getNextHighestDepth());
	mc._x = x;
	mc._y = y;
	mc.beginFill(0xFF0000);
	mc.moveTo(10, 10);
	mc.lineTo(100, 10);
	mc.lineTo(100, 100);
	mc.lineTo(10, 100);
	mc.lineTo(10, 10);
	mc.endFill();
}

function setFunctions(mc) {
	mc.onPress = function() {
		trace(this + ".onPress");
	};
	mc.onRelease = function() {
		trace(this + ".onRelease");
	};
	mc.onMouseDown = function() {
		trace(this + ".onMouseDown");
	};
	mc.onMouseUp = function() {
		trace(this + ".onMouseUp");
	};
}

if(this === _level0) {
	createMC("square", 0, 0, this);
	setFunctions(this);
	// Issue #18610.
	this.createEmptyMovieClip("mc2", this.getNextHighestDepth());
	loadMovie("test.swf", "mc2");
} else {
	this._lockroot = true;
	createMC("square", 100, 100, this);
	setFunctions(this);
}
