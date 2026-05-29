trace("//function SuperClass() {}");
function SuperClass() {
}

trace("//sub_prototype = new SuperClass();")
var sub_prototype = new SuperClass();

trace("//sub_prototype.constructor === SuperClass");
trace(sub_prototype.constructor === SuperClass);

trace("//sub_prototype.constructor === Object");
trace(sub_prototype.constructor === Object);

trace("//sub_prototype.hasOwnProperty('constructor')");
trace(sub_prototype.hasOwnProperty("constructor"));

trace("//function SubClass() {}");
function SubClass() {
}

trace("//SubClass.prototype.constructor === SubClass");
trace(SubClass.prototype.constructor === SubClass);

trace("//SubClass.prototype.constructor === SuperClass");
trace(SubClass.prototype.constructor === SuperClass);

trace("//SubClass.prototype.constructor === Object");
trace(SubClass.prototype.constructor === Object);

trace("//SubClass.prototype.hasOwnProperty('constructor')");
trace(SubClass.prototype.hasOwnProperty("constructor"));

trace("//SubClass.prototype = sub_prototype");
SubClass.prototype = sub_prototype;

trace("//SubClass.prototype.constructor === SubClass");
trace(SubClass.prototype.constructor === SubClass);

trace("//SubClass.prototype.constructor === SuperClass");
trace(SubClass.prototype.constructor === SuperClass);

trace("//SubClass.prototype.constructor === Object");
trace(SubClass.prototype.constructor === Object);

trace("//SubClass.prototype.hasOwnProperty('constructor')");
trace(SubClass.prototype.hasOwnProperty("constructor"));

trace("//sc_instance = new SubClass();");
var sc_instance = new SubClass();

trace("//sc_instance.constructor === SubClass");
trace(sc_instance.constructor === SubClass);

trace("//sc_instance.constructor === SuperClass");
trace(sc_instance.constructor === SuperClass);

trace("//sc_instance.constructor === Object");
trace(sc_instance.constructor === Object);

trace("//sc_instance.hasOwnProperty('constructor')");
trace(sc_instance.hasOwnProperty("constructor"));

fscommand("quit");
