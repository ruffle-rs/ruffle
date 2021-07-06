package {
	public class Test {
	}
}

class Superclass {
	
}

class Subclass extends Superclass {
	
}

function trace_vector(v: Vector.<*>) {
	trace(v.length, "elements");
	for (var i = 0; i < v.length; i += 1) {
		trace(v[i]);
	}
}

trace("/// var a0 = new Superclass();");
var a0 = new Superclass();

trace("/// var a1 = new Subclass();");
var a1 = new Subclass();

trace("/// var a: Vector.<Superclass> = new <Superclass>[a0, a1];");
var a:Vector.<Superclass> = new <Superclass>[a0, a1];

trace("/// var b: Vector.<Subclass> = new <Subclass>[];");
var b:Vector.<Subclass> = new <Subclass>[];

trace("/// b.length = 1;");
b.length = 1;

trace("/// b[0] = new Subclass();");
b[0] = new Subclass();

trace("/// a.pop();");
trace(a.pop());

trace("/// a.pop();");
trace(a.pop());

trace("/// a.pop();");
trace(a.pop());

trace("/// a.push(a1, a0, a0, a1);");
trace(a.push(a1, a0, a0, a1));

trace("/// a[0] === a[3];");
trace(a[0] === a[3]);

trace("/// a[1] === a[2];");
trace(a[1] === a[2]);

trace("/// (contents of a...)");
trace_vector(a);

trace("/// b.push(new Subclass());");
trace(b.push(new Subclass()));

trace("/// (contents of b...)");
trace_vector(b);