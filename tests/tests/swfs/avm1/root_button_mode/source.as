// Test made with JPEXS Free Flash Decompiler.

function createMC(name, x, y) {
	var mc = _root.createEmptyMovieClip(name, _root.getNextHighestDepth());
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

createMC("square", 0, 0);
setFunctions(_root);
