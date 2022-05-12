package {
	public class Test {}
}

var foo = function():Boolean
{
	trace("Called foo!")
	return true;
};

function dummy():void {
	trace("Called dummy!");
}

trace('// foo())')
trace(foo());
trace('// foo("A")')
trace(foo("A"));
trace('// foo(true, true, false)');
trace(foo(true, true, false));

var testing = dummy;
trace('// testing()');
trace(testing());
trace('// testing(true)');
trace(testing(true));