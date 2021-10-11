package {
	public class Test {}
}

with (Math) {
	trace(abs(-10));
}

var test = {"example": "scope test"}

with (test) {
	trace(example);
}

with (Math) {
	var f = function () {
		trace(floor(20.3))
	}	
	f();
}

with (test) {
	var f = function () {
		trace(example)
	}	
	f();
}