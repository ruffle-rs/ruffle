package {
	import flash.errors.IllegalOperationError;
	public class Test {
		public function Test() {
			var errors = [Error, RangeError, IllegalOperationError, ArgumentError, ReferenceError];
			for (var i = 0; i < errors.length; i++) {
				var cls = errors[i];
				trace("Class: " + cls);
				trace("cls.prototype.name = " + cls.prototype.name);
				var err = new cls("My Error", 42);
				trace(err.toString());
				trace(err.name);
				trace(err.errorID);
				trace();
			}
		}
	}
}