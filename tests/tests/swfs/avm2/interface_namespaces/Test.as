package {
	public class Test {
	}
}

namespace ruffle = "https://ruffle.rs/AS3/test_ns";

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
	
	ruffle function method() {
		trace("Ruffle-NS instance method");
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

interface IConflictTest2 {
	function method();
}

class ConflictTest implements ITest2, IConflictTest2 {
	public function method() {
		trace("Conflicting instance method");
	}
	
	public function method2() {
		trace("Nonconflicting instance method 2");
	}
}

trace("///var x = new Test3();");
var x = new Test3();
trace("///var x_as_iface2: ITest2 = x;");
var x_as_iface2: ITest2 = x;
trace("///var x_as_iface3: ITest3 = x;");
var x_as_iface3: ITest3 = x;

trace("///x_as_iface2.method();");
trace(x_as_iface2.method());

trace("///x_as_iface2.method2();");
trace(x_as_iface2.method2());

trace("///x_as_iface2.ruffle::method();");
trace(x_as_iface2.ruffle::method());

trace("///x_as_iface3.method();");
trace(x_as_iface3.method());

trace("///x_as_iface3.method2();");
trace(x_as_iface3.method2());

trace("///x_as_iface3.ruffle::method();");
trace(x_as_iface3.ruffle::method());

trace("///x_as_iface3.method3();");
trace(x_as_iface3.method3());

trace("///var y = new Test4();");
var y = new Test4();
trace("///var y_as_iface2: ITest2 = y;");
var y_as_iface2: ITest2 = y;
trace("///var y_as_iface3: ITest3 = y;");
var y_as_iface3: ITest3 = y;

trace("///y_as_iface2.method();");
trace(y_as_iface2.method());

trace("///y_as_iface2.method2();");
trace(y_as_iface2.method2());

trace("///y_as_iface2.ruffle::method();");
trace(y_as_iface2.ruffle::method());

trace("///y_as_iface3.method();");
trace(y_as_iface3.method());

trace("///y_as_iface3.method2();");
trace(y_as_iface3.method2());

trace("///y_as_iface3.method3();");
trace(y_as_iface3.method3());

trace("///y_as_iface3.ruffle::method();");
trace(y_as_iface3.ruffle::method());

trace("///var z = new ConflictTest();");
var z = new ConflictTest();
trace("///var z_as_iface2: ITest2 = z;");
var z_as_iface2: ITest2 = z;
trace("///var z_as_conflict: IConflictTest2 = z;");
var z_as_conflict: IConflictTest2 = z;

trace("///z_as_iface2.method();");
trace(z_as_iface2.method());

trace("///z_as_iface2.method2();");
trace(z_as_iface2.method2());

trace("///z_as_conflict.method();");
trace(z_as_conflict.method());