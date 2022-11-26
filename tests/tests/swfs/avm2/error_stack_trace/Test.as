package  {
	public class Test {
		{
				trace(new Error().getStackTrace())
		}
	
		static function class_method() {
			trace(new Error().getStackTrace())
		}
		public function Test() {
				trace(new Error().getStackTrace());
				this.my_namespace::f();
				this.getter
				this.setter = 10;
				class_method();
		}
	
		function get getter() {
				trace(new Error().getStackTrace())
				class_method();
		}
	
		function set setter(a) {
				trace(new Error().getStackTrace())
				class_method();
		}
	
		my_namespace function f(){ trace(new Error().getStackTrace())}
	}
	
}

function b() {
	var temp = function() {
		trace(new Error().getStackTrace());
	}
	temp()
	
}
b();

var e = new Error("test message", 100);
e.name = "Test";
trace(e.errorID);
trace(e.message);
trace(e.name);
trace(e);
trace(e.getStackTrace());