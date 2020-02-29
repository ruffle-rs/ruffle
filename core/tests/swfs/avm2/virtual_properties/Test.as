package {
	public class Test {
	}
}

class Test2 {
	function get prop() {
		return "Test2 Prop";
	}
	
	function set prop(val:String) {
		trace(val);
	}
	
	function set prop2(val:String) {
		trace("Test2 Set Prop2");
	}
	
	function get prop3() {
		return "Test2 Prop3";
	}
}

class Test3 extends Test2 {
	function get prop2() {
		return "Test3 Prop2";
	}
	
	function set prop3(val:String) {
		trace(val);
	}
}

class Test4 extends Test3 {
	override function get prop() {
		trace("Child Prop2 getter");
		return super.prop;
	}
	
	override function set prop(val:String) {
		trace("Child Prop2 Setter");
		super.prop = val;
	}
}

var w = new Test2();
trace(w.prop);
w.prop = "Setting Test2 Prop";

w.prop2 = "TEST FAIL - Test2::prop2 SETTER DOES NOT TRACE THIS VALUE";

trace(w.prop3);

var x = new Test3();
trace(x.prop);
x.prop = "Setting Test3 Prop";

trace(x.prop2);
x.prop3 = "Setting Test3 Prop3";

var y = new Test4();
trace(y.prop);
y.prop = "Setting Test4 Prop";

trace(y.prop2);
y.prop2 = "TEST FAIL - Test4::prop2 SETTER DOES NOT TRACE THIS VALUE";

trace(y.prop3);
y.prop3 = "Setting Test4 Prop3";