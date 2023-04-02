package {
	public class Test {
		public function example() {
			trace(this.example.length);
			trace("worked");
		}
		public function Test() {
			this.example.call(this);
		}
	}
}