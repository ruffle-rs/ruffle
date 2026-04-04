package {
	import flash.display.Sprite;
	import flash.utils.ByteArray;

	public class Test extends Sprite {
		public function Test() {
			testRoundtrip("unicode");
			testRoundtrip("utf-16");
			testRoundtrip("utf-16le");
			testRoundtrip("utf-16be");
		}

		private function testRoundtrip(encoding: String): void {
			var bytes: ByteArray = new ByteArray();
			bytes.writeMultiByte("A", encoding);
			bytes.position = 0;
			var output: String = bytes.readMultiByte(2, encoding);
			trace(encoding + " : " + output);
			trace(encoding + ".length : " + output.length);
		}
	}
}
