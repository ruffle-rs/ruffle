package {
	import flash.net.LocalConnection;

	public class CustomLocalConnection extends LocalConnection {
		private var main: Test;
		
		public function CustomLocalConnection(main: Test) {
			super();
			this.main = main;
		}
		
		public function test() {
			trace("custom.test was called with " + arguments.length + " argument" + (arguments.length == 0 ? "" : "s"));
			if (arguments.length > 0) {
				trace("  " + main.repr(arguments));
			}
		}

		public function throwAnError() {
			trace("custom.throwAnError was called");
			//throw "aah!"; // [NA] this crashes every Flash Player I've tried
			//throw {}; // [NA] this causes an error when constructing the AsyncErrorEvent
			//throw new Error("aaah!");
		}
	}
}
