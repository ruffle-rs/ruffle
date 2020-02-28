package {
	public class Test {
	}
}

class Test2 {
	{
		trace("Class constructor");
	}
	
	function Test2() {
		trace("Instance constructor");
	}
	
	function method() {
		trace("Instance method");
	}
	
	function method2() {
		trace("Instance method 2");
	}
}

class Test3 extends Test2 {
	{
		trace("Child class constructor");
	}
	
	function Test3() {
		trace("Child instance constructor pre-super");
		super();
		trace("Child instance constructor post-super");
	}
	
	override function method() {
		trace("Child instance method pre-super");
		super.method();
		trace("Child instance method post-super");
	}
	
	function method3() {
		trace("Child instance method3 pre-super");
		super.method();
		trace("Child instance method3 post-super");
	}
}

class Test4 extends Test3 {
	{
		trace("Grandchild class constructor");
	}
	
	function Test4() {
		trace("Grandchild instance constructor pre-super");
		super();
		trace("Grandchild instance constructor post-super");
	}
	
	override function method2() {
		trace("Grandchild instance method2 pre-super");
		super.method2();
		trace("Grandchild instance method2 post-super");
	}
	
	override function method3() {
		trace("Grandchild instance method3 pre-super");
		super.method3();
		trace("Grandchild instance method3 post-super");
	}
}

trace("Script initializer");
var x = new Test3();
x.method();
x.method2();
x.method3();

var y = new Test4();
y.method();
y.method2();
y.method3();