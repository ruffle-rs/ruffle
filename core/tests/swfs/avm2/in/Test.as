package {
	public class Test {}
}

class ES4Class extends Object {
	public var test_var = "var";
	public const test_const = "const";

	public function test_function() {
		trace("test_function");
	}

	public function get test_virt() {
		return "test_virt";
	}

	public function set test_virt(val) {
		trace("test_virt");
	}

	public static var test_static_var = "var";
	public static const test_static_const = "const";

	public static function test_static_function() {
		trace("test_static_function");
	}

	public static function get test_static_virt() {
		return "test_static_virt";
	}

	public static function set test_static_virt(val) {
		trace("test_static_virt");
	}
	
	private var test_private_var = "private_var";
	private const test_private_const = "private_const";
	
	private function test_private_function() {
		trace("test_private_function");
	}

	private function get test_private_virt() {
		return "test_private_virt";
	}

	private function set test_private_virt(val) {
		trace("test_private_virt");
	}
}

function ES3Class() {
	this.test_var = "var";
}

ES3Class.test_static_var = "var";

ES3Class.test_static_function = function () {
	trace("test_static_function");
}

ES3Class.prototype.test_function = function() {
	trace("test_function");
}

ES3Class.prototype.test_proto = "proto_var";

var es4inst = new ES4Class();
var es3inst = new ES3Class();

trace("//'test_var' in es4inst");
trace('test_var' in es4inst);
trace("//'test_const' in es4inst");
trace('test_const' in es4inst);
trace("//'test_function' in es4inst");
trace('test_function' in es4inst);
trace("//'test_virt' in es4inst");
trace('test_virt' in es4inst);
trace("//'test_static_var' in es4inst");
trace('test_static_var' in es4inst);
trace("//'test_static_const' in es4inst");
trace('test_static_const' in es4inst);
trace("//'test_static_function' in es4inst");
trace('test_static_function' in es4inst);
trace("//'test_static_virt' in es4inst");
trace('test_static_virt' in es4inst);
trace("//'test_private_var' in es4inst");
trace('test_private_var' in es4inst);
trace("//'test_private_const' in es4inst");
trace('test_private_const' in es4inst);
trace("//'test_private_function' in es4inst");
trace('test_private_function' in es4inst);
trace("//'test_private_virt' in es4inst");
trace('test_private_virt' in es4inst);

trace("//'test_var' in ES4Class");
trace('test_var' in ES4Class);
trace("//'test_const' in ES4Class");
trace('test_const' in ES4Class);
trace("//'test_function' in ES4Class");
trace('test_function' in ES4Class);
trace("//'test_virt' in ES4Class");
trace('test_virt' in ES4Class);
trace("//'test_static_var' in ES4Class");
trace('test_static_var' in ES4Class);
trace("//'test_static_const' in ES4Class");
trace('test_static_const' in ES4Class);
trace("//'test_static_function' in ES4Class");
trace('test_static_function' in ES4Class);
trace("//'test_static_virt' in ES4Class");
trace('test_static_virt' in ES4Class);
trace("//'test_private_var' in ES4Class");
trace('test_private_var' in ES4Class);
trace("//'test_private_const' in ES4Class");
trace('test_private_const' in ES4Class);
trace("//'test_private_function' in ES4Class");
trace('test_private_function' in ES4Class);
trace("//'test_private_virt' in ES4Class");
trace('test_private_virt' in ES4Class);

trace("//'test_var' in ES4Class.prototype");
trace('test_var' in ES4Class.prototype);
trace("//'test_const' in ES4Class.prototype");
trace('test_const' in ES4Class.prototype);
trace("//'test_function' in ES4Class.prototype");
trace('test_function' in ES4Class.prototype);
trace("//'test_virt' in ES4Class.prototype");
trace('test_virt' in ES4Class.prototype);
trace("//'test_static_var' in ES4Class.prototype");
trace('test_static_var' in ES4Class.prototype);
trace("//'test_static_const' in ES4Class.prototype");
trace('test_static_const' in ES4Class.prototype);
trace("//'test_static_function' in ES4Class.prototype");
trace('test_static_function' in ES4Class.prototype);
trace("//'test_static_virt' in ES4Class.prototype");
trace('test_static_virt' in ES4Class.prototype);
trace("//'test_private_var' in ES4Class.prototype");
trace('test_private_var' in ES4Class.prototype);
trace("//'test_private_const' in ES4Class.prototype");
trace('test_private_const' in ES4Class.prototype);
trace("//'test_private_function' in ES4Class.prototype");
trace('test_private_function' in ES4Class.prototype);
trace("//'test_private_virt' in ES4Class.prototype");
trace('test_private_virt' in ES4Class.prototype);

trace("//'test_var' in es3inst");
trace('test_var' in es3inst);
trace("//'test_function' in es3inst");
trace('test_function' in es3inst);
trace("//'test_proto' in es3inst");
trace('test_proto' in es3inst);
trace("//'test_static_var' in es3inst");
trace('test_static_var' in es3inst);
trace("//'test_static_function' in es3inst");
trace('test_static_function' in es3inst);

trace("//'test_var' in ES3Class");
trace('test_var' in ES3Class);
trace("//'test_function' in ES3Class");
trace('test_function' in ES3Class);
trace("//'test_proto' in ES3Class");
trace('test_proto' in ES3Class);
trace("//'test_static_var' in ES3Class");
trace('test_static_var' in ES3Class);
trace("//'test_static_function' in ES3Class");
trace('test_static_function' in ES3Class);

trace("//'test_var' in ES3Class.prototype");
trace('test_var' in ES3Class.prototype);
trace("//'test_function' in ES3Class.prototype");
trace('test_function' in ES3Class.prototype);
trace("//'test_proto' in ES3Class.prototype");
trace('test_proto' in ES3Class.prototype);
trace("//'test_static_var' in ES3Class.prototype");
trace('test_static_var' in ES3Class.prototype);
trace("//'test_static_function' in ES3Class.prototype");
trace('test_static_function' in ES3Class.prototype);