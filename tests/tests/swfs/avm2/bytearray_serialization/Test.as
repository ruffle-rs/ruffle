package {
	import flash.utils.ByteArray;
  import flash.display.Sprite;

	public class Test extends Sprite {
    private function PrintByteArray(name: String, data:ByteArray):void {
      var bytes: Array = []
			for (var i: int = 0; i < data.length; i++) {
				bytes.push(data.readUnsignedByte());
			}
			trace(name + ": " + bytes);
      data.position = 0;
    }

		public function Test() {

      var original:ByteArray = new ByteArray();
      original.writeUTF("hello world");
      original.position = 0;

      PrintByteArray("Original", original);

			var serialized:ByteArray = new ByteArray();
			serialized.writeObject(original);
			serialized.position = 0;

			PrintByteArray("Serialized", serialized);

			var readBack:ByteArray = serialized.readObject();

      PrintByteArray("ReadBack", readBack);
		}
	}
}
