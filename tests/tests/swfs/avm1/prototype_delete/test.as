trace("// Deleting Object.prototype");
trace(Object.prototype);
delete Object.prototype;
trace(Object.prototype);

trace("// Deleting MovieClip.prototype");
trace(MovieClip.prototype);
delete MovieClip.prototype;
trace(MovieClip.prototype);

class Base {

}

class Extended extends Base {

}

trace("// Deleting Base.prototype");
trace(Base.prototype);
delete Base.prototype;
trace(Base.prototype);

trace("// Deleting Extended.prototype");
trace(Extended.prototype);
delete Extended.prototype;
trace(Extended.prototype);
