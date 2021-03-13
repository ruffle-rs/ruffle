package {
	public class Test {
	}
}

class Superclass {
	
}

class Subclass extends Superclass {
	
}

trace("/// var a: Vector.<Superclass> = new <Superclass>[];");
var a:Vector.<Superclass> = new <Superclass>[];

trace("/// a.length = 2;");
a.length = 2;

trace(a[0]);
trace(a[1]);

trace("/// a[0] = new Superclass();");
a[0] = new Superclass();

trace("/// a[1] = new Subclass();");
a[1] = new Subclass();

trace(a[0]);
trace(a[1]);