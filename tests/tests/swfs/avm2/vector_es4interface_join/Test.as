package {
	public class Test {
	}
}

interface ISubclass {
	
}

class Subclass implements ISubclass {
	
}

function trace_vec(v) {
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a: Vector.<ISubclass> = new <ISubclass>[];");
var a:Vector.<ISubclass> = new <ISubclass>[];

trace("/// a.length = 1;");
a.length = 1;

trace("/// a[0] = new Subclass();");
a[0] = new Subclass();

trace("/// var b: Vector.<Subclass> = new <Subclass>[];");
var b:Vector.<Subclass> = new <Subclass>[];

trace("/// b.length = 2;");
b.length = 2;

trace("/// b[0] = new Subclass();");
b[0] = new Subclass();

trace("/// b[1] = new Subclass();");
b[1] = new Subclass();

trace("/// a.join('...');");
trace(a.join("..."));

trace("/// b.join('...');");
trace(b.join("..."));