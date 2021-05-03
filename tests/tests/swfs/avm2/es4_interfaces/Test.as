package {
	public class Test {
	}
}

interface ITest2 {
	{
		trace("TEST FAIL: ITest2 class constructor should not run");
	}
	
	function method();
	function method2();
}

class Test2 implements ITest2 {
	{
		trace("Class constructor");
	}
	
	function Test2() {
		trace("Instance constructor");
	}
	
	public function method() {
		trace("Instance method");
	}
	
	public function method2() {
		trace("Instance method 2");
	}
}

interface ITest3 extends ITest2 {
	{
		trace("TEST FAIL: ITest3 class constructor should not run");
	}
	
	function method3()
}

class Test3 extends Test2 implements ITest3 {
	{
		trace("Child class constructor");
	}
	
	function Test3() {
		trace("Child instance constructor pre-super");
		super();
		trace("Child instance constructor post-super");
	}
	
	public override function method() {
		trace("Child instance method pre-super");
		super.method();
		trace("Child instance method post-super");
	}
	
	public function method3() {
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
	
	public override function method2() {
		trace("Grandchild instance method2 pre-super");
		super.method2();
		trace("Grandchild instance method2 post-super");
	}
	
	public override function method3() {
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