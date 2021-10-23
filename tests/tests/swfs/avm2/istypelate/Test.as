package {
	public class Test {
	}
}

interface ITest2 {
	function method();
	function method2();
}

class Test2 implements ITest2 {
	function Test2() {
	}
	
	public function method() {
		trace("Instance method");
	}
	
	public function method2() {
		trace("Instance method 2");
	}
}

interface ITest3 extends ITest2 {
	function method3()
}

class Test3 extends Test2 implements ITest3 {
	function Test3() {
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
	function Test4() {
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

var x = new Test3();

trace("//x is Object");
trace(x is Object);

trace("//x is Test2");
trace(x is Test2);

trace("//x is ITest2");
trace(x is ITest2);

trace("//x is Test3");
trace(x is Test3);

trace("//x is ITest3");
trace(x is ITest3);

trace("//x is Test4");
trace(x is Test4);

var y = new Test4();

trace("//y is Object");
trace(y is Object);

trace("//y is Test2");
trace(y is Test2);

trace("//y is ITest2");
trace(y is ITest2);

trace("//y is Test3");
trace(y is Test3);

trace("//y is ITest3");
trace(y is ITest3);

trace("//y is Test4");
trace(y is Test4);

trace("//Test3.prototype is Object");
trace(Test3.prototype is Object);

trace("//Test3.prototype is Test2");
trace(Test3.prototype is Test2);

trace("//Test3.prototype is ITest2");
trace(Test3.prototype is ITest2);

trace("//Test3.prototype is Test3");
trace(Test3.prototype is Test3);

trace("//Test3.prototype is ITest3");
trace(Test3.prototype is ITest3);

trace("//Test3.prototype is Test4");
trace(Test3.prototype is Test4);

trace("//Test4.prototype is Object");
trace(Test4.prototype is Object);

trace("//Test4.prototype is Test2");
trace(Test4.prototype is Test2);

trace("//Test4.prototype is ITest2");
trace(Test4.prototype is ITest2);

trace("//Test4.prototype is Test3");
trace(Test4.prototype is Test3);

trace("//Test4.prototype is ITest3");
trace(Test4.prototype is ITest3);

trace("//Test4.prototype is Test4");
trace(Test4.prototype is Test4);

trace("//Object.prototype is Object");
trace(Object.prototype is Object);

trace("//Function.prototype is Object");
trace(Function.prototype is Object);

trace("//Function.prototype is Function");
trace(Function.prototype is Function);

trace("//Class.prototype is Object");
trace(Class.prototype is Object);

trace("//Class.prototype is Class");
trace(Class.prototype is Class);