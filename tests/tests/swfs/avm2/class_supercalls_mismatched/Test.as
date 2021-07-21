package {
	public class Test {
	}
}

class Base {
	var a_prop: int = 1;
	
	function set b_base(value) {
		trace("// (Base.b_base set with " + value + ")");
		
		return "value of b_base";
	}
	
	function get c_base() {
		trace("// (Base.c_base get)");
		
		return "value of c_base";
	}
	
	function get d_base() {
		trace("//(Base.d_base called)");
		return function() {
			trace("// (Base.d_base closure called)");
		}
	}
	
	var e_base = function() {
		trace("//Base.e_base function called)");
	}
}

dynamic class Evil extends Base {
	function get a() {
		trace("//(Evil.a get)");
		return super.a_prop;
	}
	
	function set a(value: int) {
		trace("//(Evil.a set with " + value + ")");
		super.a_prop = value;
	}
	
	function get b() {
		trace("//(Evil.b get)");
		return super.b_base = 20;
	}
	
	function set c(value) {
		trace("//(Evil.c set with " + value + ")");
		super.c_base = value;
		
		return super.c_base;
	}
	
	function d() {
		super.d_base();
	}
	
	function e() {
		super.e_base();
	}
}

trace("//var e = new Evil()");
var e = new Evil();

trace("//e.a");
trace(e.a);

trace("//e.a = 10");
trace(e.a = 10);

trace("//e.a");
trace(e.a);

trace("//e.b");
trace(e.b);

trace("//e.c = 14");
trace(e.c = 14);

trace("//e.d()");
trace(e.d());

trace("//e.e()");
trace(e.e());

trace("//e.e_base = (a different function...)");
e.e_base = function() {
	trace("// (Patched function e_base called!)");
	trace(this);
}

trace("//e.e()");
trace(e.e());