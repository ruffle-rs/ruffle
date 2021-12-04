package {
	public class Test {}
}

class ES4Class extends Object {
	public var value: String;
	
	public function ES4Class(value: String) {
		this.value = value;
	}
	
	public function test_method() {
		trace(this.value);
	}
}

var x = new ES4Class("var x: ES4Class");
var y = new ES4Class("var y: ES4Class");

trace("Using ES4 Class method...");
x.test_method.call(y);
y.test_method.call(x);

trace("Using prototype method...");

ES4Class.prototype.test_proto_method = function () {
	trace(this.value);
}

x.test_proto_method.call(y);
y.test_proto_method.call(x);

(function () {
	trace("Hi");
	trace(this);
}).call.call();
