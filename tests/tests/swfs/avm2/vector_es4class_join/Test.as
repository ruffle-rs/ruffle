package {
	public class Test {
	}
}

class Superclass {
	
}

class Subclass extends Superclass {
	
}

function trace_vec(v) {
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a: Vector.<Superclass> = new <Superclass>[];");
var a:Vector.<Superclass> = new <Superclass>[];

trace("/// a.length = 2;");
a.length = 2;

trace("/// a[0] = new Superclass();");
a[0] = new Superclass();

trace("/// a[1] = new Subclass();");
a[1] = new Subclass();

trace("/// var b: Vector.<Subclass> = new <Subclass>[];");
var b:Vector.<Subclass> = new <Subclass>[];

trace("/// b.length = 1;");
b.length = 1;

trace("/// b[0] = new Subclass();");
b[0] = new Subclass();

trace("/// a.join('...');");
trace(a.join("..."));

trace("/// b.join('...');");
trace(b.join("..."));