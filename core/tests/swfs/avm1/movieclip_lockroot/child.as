function setLock(lock) {
	this._lockroot = lock;
}

function test(lock) {
	trace("Child: level is " + this + ", root is " + _root + ", _lockroot is " + this._lockroot + ", parent is " + this._parent);
}
