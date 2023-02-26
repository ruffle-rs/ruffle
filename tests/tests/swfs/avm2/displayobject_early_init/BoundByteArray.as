package {
	import flash.utils.ByteArray;

	[Embed(source="data.txt", mimeType="application/octet-stream")]
	public class BoundByteArray extends ByteArray {		
		public function BoundByteArray() {
			trace("BoundByteArray before super(): this.bytesAvailable = " + this.bytesAvailable);
			super();
			trace("BoundByteArray after super(): this.bytseAvailable = " + this.bytesAvailable);			
		}
	}
}