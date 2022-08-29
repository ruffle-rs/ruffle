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

trace("//null as Object");
trace(null as Object);

trace("//undefied as Object");
trace(undefined as Object);

trace("//x as Object");
trace(x as Object);

trace("//x as Test2");
trace(x as Test2);

trace("//x as ITest2");
trace(x as ITest2);

trace("//x as Test3");
trace(x as Test3);

trace("//x as ITest3");
trace(x as ITest3);

trace("//x as Test4");
trace(x as Test4);

var y = new Test4();

trace("//y as Object");
trace(y as Object);

trace("//y as Test2");
trace(y as Test2);

trace("//y as ITest2");
trace(y as ITest2);

trace("//y as Test3");
trace(y as Test3);

trace("//y as ITest3");
trace(y as ITest3);

trace("//y as Test4");
trace(y as Test4);