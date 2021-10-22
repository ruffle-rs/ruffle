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

trace("//x instanceof Object");
trace(x instanceof Object);

trace("//x instanceof Test2");
trace(x instanceof Test2);

trace("//x instanceof ITest2");
trace(x instanceof ITest2);

trace("//x instanceof Test3");
trace(x instanceof Test3);

trace("//x instanceof ITest3");
trace(x instanceof ITest3);

trace("//x instanceof Test4");
trace(x instanceof Test4);

var y = new Test4();

trace("//y instanceof Object");
trace(y instanceof Object);

trace("//y instanceof Test2");
trace(y instanceof Test2);

trace("//y instanceof ITest2");
trace(y instanceof ITest2);

trace("//y instanceof Test3");
trace(y instanceof Test3);

trace("//y instanceof ITest3");
trace(y instanceof ITest3);

trace("//y instanceof Test4");
trace(y instanceof Test4);

trace("//Test3.prototype instanceof Object");
trace(Test3.prototype instanceof Object);

trace("//Test3.prototype instanceof Test2");
trace(Test3.prototype instanceof Test2);

trace("//Test3.prototype instanceof ITest2");
trace(Test3.prototype instanceof ITest2);

trace("//Test3.prototype instanceof Test3");
trace(Test3.prototype instanceof Test3);

trace("//Test3.prototype instanceof ITest3");
trace(Test3.prototype instanceof ITest3);

trace("//Test3.prototype instanceof Test4");
trace(Test3.prototype instanceof Test4);

trace("//Test4.prototype instanceof Object");
trace(Test4.prototype instanceof Object);

trace("//Test4.prototype instanceof Test2");
trace(Test4.prototype instanceof Test2);

trace("//Test4.prototype instanceof ITest2");
trace(Test4.prototype instanceof ITest2);

trace("//Test4.prototype instanceof Test3");
trace(Test4.prototype instanceof Test3);

trace("//Test4.prototype instanceof ITest3");
trace(Test4.prototype instanceof ITest3);

trace("//Test4.prototype instanceof Test4");
trace(Test4.prototype instanceof Test4);