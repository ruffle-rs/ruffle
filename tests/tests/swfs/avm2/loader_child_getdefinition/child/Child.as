package {
	public class Child extends Parent {
		public function Child() {
			trace("Calling super()");
			super();
		}
	}
}