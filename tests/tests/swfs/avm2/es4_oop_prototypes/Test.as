// compiled with mxmlc

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {
        }
    }
}

class ES4Class extends Object {
	public function ES4Class() {
		trace("ES4Class constructor");
	}
	
	public function test_method() {
		trace("ES4Class test_method");
	}
	
	static public function test_static() {
		trace("ES4Class test_static");
	}
	
	public var test_var = "ES4Class test_var";
	
	public const test_const = "ES4Class test_const";
	
	public function get test_get() {
		return "ES4Class test_get";
	}
}

ES4Class.prototype.test_proto_var = "ES4Class test_proto_var";
ES4Class.prototype.test_var = "TEST FAIL: Class variables override prototype variables!";
ES4Class.prototype.test_const = "TEST FAIL: Class constants override prototype variables!";

ES4Class.prototype.test_proto_method = function () {
	trace("ES4Class test_proto_method");
};

ES4Class.prototype.test_method = function () {
	trace("TEST FAIL: Class methods override prototype functions!");
};

ES4Class.prototype.test_static = function () {
	trace("TEST FAIL: Class static methods override prototype functions!");
};

ES4Class.prototype.test_get = "TEST FAIL: Class getters override prototype properties!";

var x = new ES4Class();

trace(x.test_var);
trace(x.test_const);
trace(x.test_proto_var);
x.test_method();
ES4Class.test_static();
x.test_proto_method();
trace(x.test_get);

trace("-----------------")

class SealedClass{}
SealedClass.prototype.field = 1;
x = new SealedClass();
trace(x.field);
try {
	trace(x.nofield);
} catch (e) {
	trace("Caught missing field")
	trace(e);
}

dynamic class DynamicClass{}
DynamicClass.prototype.field = 1;
x = new DynamicClass();
trace(x.field);
trace(x.nofield);

