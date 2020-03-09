package {
	public class Test {}
}

var freeFunction = function() {
};

function namedFunction() {
	
}

trace("//freeFunction.valueOf() === freeFunction");
trace(freeFunction.valueOf() === freeFunction);

trace("//namedFunction.valueOf() === namedFunction");
trace(namedFunction.valueOf() === namedFunction);