package {
	public class Test {
		public function Test() {
			trace("'foo'.hasOwnProperty('toString'): " + 'foo'.hasOwnProperty('toString'));
			trace("'foo'.hasOwnProperty('charAt'): " + 'foo'.hasOwnProperty('charAt'));
			trace("true.hasOwnProperty('toString'): " + true.hasOwnProperty('toString'));
			trace("(20).hasOwnProperty('toString'): " + (20).hasOwnProperty('toString'));
			trace("(1.5).hasOwnProperty('toString'): " + (1.5).hasOwnProperty('toString'));
			trace("true.propertyIsEnumerable('dummy'): " + true.propertyIsEnumerable('dummy'));
		}
	}
}
