function testHit() {
	var bounds = hitbox.getBounds();
	trace("Bounds:");
	for(var i in bounds) {
		trace("\t" + i + ": " + String(bounds[i]));
	}
	var rect = hitbox.getRect();
	trace("Rectangle:");
	for(var i in rect) {
		trace("\t" + i + ": " + String(rect[i]));
	}
	
	trace("hitTest:" + bullet.hitTest(hitbox));
	trace("hitTestPoint:" + bullet.hitTest(80, 30, true));
	trace("hitTestPoint Non-Shape:" + bullet.hitTest(80, 30, false));
}

trace("// frame1");
testHit();

trace("// hitbox.gotoAndStop(2);");
hitbox.gotoAndStop(2);
testHit();

trace("// hitbox.gotoAndStop(3);");
hitbox.gotoAndStop(3);
testHit();

trace("// hitbox.gotoAndStop(4);");
hitbox.gotoAndStop(4);
testHit();

trace("// hitbox.gotoAndStop(5);");
hitbox.gotoAndStop(5);
testHit();