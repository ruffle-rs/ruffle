package 
{
	import flash.utils.ByteArray;
	public class Test
	{
		var TESTS = [
		undefined,
		null,
		false,
		true,
		4,
		4.5,
		Infinity,
		-Infinity,
		NaN,
		"test"
		];
		
		public function testValue(value) {
			var ba = new ByteArray();
			ba.writeObject(value);
			ba.position = 0;
			trace(ba.readObject());
		}
		public function runTests() {
			for each(var val in TESTS) {
				testValue(val);
			}
			testValue(TESTS);
		}
		public function Test()
		{
			trace("AMF3 TESTS");
			runTests();
			ByteArray.defaultObjectEncoding = 0;
			trace("AMF0 TESTS");
			runTests();
		}
	}
}