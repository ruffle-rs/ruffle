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

trace("/// var a0 = new Superclass();");
var a0 = new Superclass();

trace("/// a[0] = a0;");
a[0] = a0;

trace("/// var a1 = new Subclass();");
var a1 = new Subclass();

trace("/// a[1] = a1;");
a[1] = a1;

trace("/// var b: Vector.<Subclass> = new <Subclass>[];");
var b:Vector.<Subclass> = new <Subclass>[];

trace("/// b.length = 1;");
b.length = 1;

trace("/// var b0 = new Subclass();");
var b0 = new Subclass();

trace("/// b[0] = b0;");
b[0] = b0;

trace("/// a.indexOf(a0);");
trace(a.indexOf(a0));

trace("/// a.indexOf(a1);");
trace(a.indexOf(a1));

trace("/// a.indexOf(b0);");
trace(a.indexOf(b0));

trace("/// b.indexOf(a0);");
trace(b.indexOf(a0));

trace("/// b.indexOf(a1);");
trace(b.indexOf(a1));

trace("/// b.indexOf(b0);");
trace(b.indexOf(b0));