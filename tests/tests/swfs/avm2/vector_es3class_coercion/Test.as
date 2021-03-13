package {
	public class Test {
	}
}

function Superclass() {
	
}

function Subclass() {
	
}

Subclass.prototype = new Superclass();

trace("/// var a: Vector.<Object> = new <Object>[];");
var a:Vector.<Object> = new <Object>[];

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