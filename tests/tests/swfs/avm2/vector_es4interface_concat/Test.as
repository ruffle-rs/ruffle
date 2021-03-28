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

trace("/// b.length = 1;");
b.length = 1;

trace("/// b[0] = new Subclass();");
b[0] = new Subclass();

trace("/// var c = a.concat(b);");
var c = a.concat(b);

trace("/// (contents of c...)");
trace_vec(c);