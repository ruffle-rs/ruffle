package {
	public class Test {
		private var func:Function = new Function();
		
		public function Test() {
			trace("Result: " + this.func(1, 2, 3, "Hi", null, new Object()));
			
			var newFunc = new Function();
			trace("Result: " + newFunc("a", [1, 2]));
		}
	}
}