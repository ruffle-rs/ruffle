package {
	public class Test {}
}

class ES4Class extends Object {
	
}

function ES3Class() {
	
}

var es4inst = new ES4Class();
var es3inst = new ES3Class();

trace("//ES4Class.prototype.isPrototypeOf(es4inst);");
trace(ES4Class.prototype.isPrototypeOf(es4inst));
trace("//Object.prototype.isPrototypeOf(es4inst);");
trace(Object.prototype.isPrototypeOf(es4inst));
trace("//ES3Class.prototype.isPrototypeOf(es4inst);");
trace(ES3Class.prototype.isPrototypeOf(es4inst));

trace("//ES4Class.prototype.isPrototypeOf(es3inst);");
trace(ES4Class.prototype.isPrototypeOf(es3inst));
trace("//Object.prototype.isPrototypeOf(es3inst);");
trace(Object.prototype.isPrototypeOf(es3inst));
trace("//ES3Class.prototype.isPrototypeOf(es3inst);");
trace(ES3Class.prototype.isPrototypeOf(es3inst));