package {
	public class Test {}
}

dynamic class ES4Class extends Object {
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

trace("//es4inst.propertyIsEnumerable('test_var')");
trace(es4inst.propertyIsEnumerable('test_var'));
trace("//es4inst.setPropertyIsEnumerable('test_var', true)");
es4inst.setPropertyIsEnumerable('test_var', true);
trace("//es4inst.propertyIsEnumerable('test_var')");
trace(es4inst.propertyIsEnumerable('test_var'));

trace("//es4inst.propertyIsEnumerable('test_const')");
trace(es4inst.propertyIsEnumerable('test_const'));
trace("//es4inst.setPropertyIsEnumerable('test_const', true)");
es4inst.setPropertyIsEnumerable('test_const', true);
trace("//es4inst.propertyIsEnumerable('test_const')");
trace(es4inst.propertyIsEnumerable('test_const'));

trace("//es4inst.propertyIsEnumerable('test_function')");
trace(es4inst.propertyIsEnumerable('test_function'));
trace("//es4inst.setPropertyIsEnumerable('test_function', true)");
es4inst.setPropertyIsEnumerable('test_function', true);
trace("//es4inst.propertyIsEnumerable('test_function')");
trace(es4inst.propertyIsEnumerable('test_function'));

trace("//es4inst.propertyIsEnumerable('test_virt')");
trace(es4inst.propertyIsEnumerable('test_virt'));
trace("//es4inst.setPropertyIsEnumerable('test_virt', true)");
es4inst.setPropertyIsEnumerable('test_virt', true);
trace("//es4inst.propertyIsEnumerable('test_virt')");
trace(es4inst.propertyIsEnumerable('test_virt'));

trace("//es4inst.propertyIsEnumerable('test_private_var')");
trace(es4inst.propertyIsEnumerable('test_private_var'));
trace("//es4inst.setPropertyIsEnumerable('test_private_var', true)");
es4inst.setPropertyIsEnumerable('test_private_var', true);
trace("//es4inst.propertyIsEnumerable('test_private_var')");
trace(es4inst.propertyIsEnumerable('test_private_var'));

trace("//es4inst.propertyIsEnumerable('test_private_const')");
trace(es4inst.propertyIsEnumerable('test_private_const'));
trace("//es4inst.setPropertyIsEnumerable('test_private_const', true)");
es4inst.setPropertyIsEnumerable('test_private_const', true);
trace("//es4inst.propertyIsEnumerable('test_private_const')");
trace(es4inst.propertyIsEnumerable('test_private_const'));

trace("//es4inst.propertyIsEnumerable('test_private_function')");
trace(es4inst.propertyIsEnumerable('test_private_function'));
trace("//es4inst.setPropertyIsEnumerable('test_private_function', true)");
es4inst.setPropertyIsEnumerable('test_private_function', true);
trace("//es4inst.propertyIsEnumerable('test_private_function')");
trace(es4inst.propertyIsEnumerable('test_private_function'));

trace("//es4inst.propertyIsEnumerable('test_private_virt')");
trace(es4inst.propertyIsEnumerable('test_private_virt'));
trace("//es4inst.setPropertyIsEnumerable('test_private_virt', true)");
es4inst.setPropertyIsEnumerable('test_private_virt', true);
trace("//es4inst.propertyIsEnumerable('test_private_virt')");
trace(es4inst.propertyIsEnumerable('test_private_virt'));

trace("//ES4Class.propertyIsEnumerable('test_static_var')");
trace(ES4Class.propertyIsEnumerable('test_static_var'));
trace("//ES4Class.setPropertyIsEnumerable('test_static_var', true)");
ES4Class.setPropertyIsEnumerable('test_static_var', true);
trace("//ES4Class.propertyIsEnumerable('test_static_var')");
trace(ES4Class.propertyIsEnumerable('test_static_var'));

trace("//ES4Class.propertyIsEnumerable('test_static_const')");
trace(ES4Class.propertyIsEnumerable('test_static_const'));
trace("//ES4Class.setPropertyIsEnumerable('test_static_const', true)");
ES4Class.setPropertyIsEnumerable('test_static_const', true);
trace("//ES4Class.propertyIsEnumerable('test_static_const')");
trace(ES4Class.propertyIsEnumerable('test_static_const'));

trace("//ES4Class.propertyIsEnumerable('test_static_function')");
trace(ES4Class.propertyIsEnumerable('test_static_function'));
trace("//ES4Class.setPropertyIsEnumerable('test_static_function', true)");
ES4Class.setPropertyIsEnumerable('test_static_function', true);
trace("//ES4Class.propertyIsEnumerable('test_static_function')");
trace(ES4Class.propertyIsEnumerable('test_static_function'));

trace("//ES4Class.propertyIsEnumerable('test_static_virt')");
trace(ES4Class.propertyIsEnumerable('test_static_virt'));
trace("//ES4Class.setPropertyIsEnumerable('test_static_virt', true)");
ES4Class.setPropertyIsEnumerable('test_static_virt', true);
trace("//ES4Class.propertyIsEnumerable('test_static_virt')");
trace(ES4Class.propertyIsEnumerable('test_static_virt'));

trace("//es3inst.propertyIsEnumerable('test_var')");
trace(es3inst.propertyIsEnumerable('test_var'));
trace("//es3inst.setPropertyIsEnumerable('test_var', false)");
es3inst.setPropertyIsEnumerable('test_var', false);
trace("//es3inst.propertyIsEnumerable('test_var')");
trace(es3inst.propertyIsEnumerable('test_var'));

trace("//ES3Class.propertyIsEnumerable('test_static_var')");
trace(ES3Class.propertyIsEnumerable('test_static_var'));
trace("//ES3Class.setPropertyIsEnumerable('test_static_var', false)");
ES3Class.setPropertyIsEnumerable('test_static_var', false);
trace("//ES3Class.propertyIsEnumerable('test_static_var')");
trace(ES3Class.propertyIsEnumerable('test_static_var'));

trace("//ES3Class.propertyIsEnumerable('test_static_function')");
trace(ES3Class.propertyIsEnumerable('test_static_function'));
trace("//ES3Class.setPropertyIsEnumerable('test_static_function', false)");
ES3Class.setPropertyIsEnumerable('test_static_function', false);
trace("//ES3Class.propertyIsEnumerable('test_static_function')");
trace(ES3Class.propertyIsEnumerable('test_static_function'));

trace("//ES3Class.prototype.propertyIsEnumerable('test_function')");
trace(ES3Class.prototype.propertyIsEnumerable('test_function'));
trace("//ES3Class.prototype.setPropertyIsEnumerable('test_function', false)");
ES3Class.prototype.setPropertyIsEnumerable('test_function', false);
trace("//ES3Class.prototype.propertyIsEnumerable('test_function')");
trace(ES3Class.prototype.propertyIsEnumerable('test_function'));

trace("//ES3Class.prototype.propertyIsEnumerable('test_proto')");
trace(ES3Class.prototype.propertyIsEnumerable('test_proto'));
trace("//ES3Class.prototype.setPropertyIsEnumerable('test_proto', false)");
ES3Class.prototype.setPropertyIsEnumerable('test_proto', false);
trace("//ES3Class.prototype.propertyIsEnumerable('test_proto')");
trace(ES3Class.prototype.propertyIsEnumerable('test_proto'));