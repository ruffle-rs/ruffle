package {
	public class Test {}
}

function closure_ab(a_start) {
	var a = a_start;
	
	return function (b_start) {
		var b = b_start;
		
		return function (c, d) {
			a += c;
			b += d;
			
			return a * b;
		}
	}
}

trace("/// var fn1 = closure_ab(1)(1);");
var fn1 = closure_ab(1)(1);

trace("/// fn1(0, 0); ");
trace(fn1(0,0));

trace("/// fn1(5, 3); ");
trace(fn1(5,3));

trace("/// var fn2 = closure_ab(3)(2);");
var fn2 = closure_ab(3)(2);

trace("/// fn2(1,1); ");
trace(fn2(1,1));

trace("/// fn1(-3,-5); ");
trace(fn1(-3,-5));

trace("/// fn2(1,1); ");
trace(fn2(1,1));